// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

use std::{
    cmp,
    collections::HashMap,
    pin::Pin,
    sync::{
        atomic,
        atomic::{AtomicBool, AtomicUsize},
        Arc,
    },
    task::{Context, Poll},
};

use ethereum_types::{Address, U256};
use futures::channel::oneshot;
use parity_runtime::Executor;

/// Manages currently reserved and prospective nonces for multiple senders.
#[derive(Debug)]
pub struct Reservations {
    nonces: HashMap<Address, SenderReservations>,
    executor: Executor,
}

impl Reservations {
    const CLEAN_AT: usize = 512;

    #[must_use]
    /// Creates a new `Reservations` instance with the given async executor.
    pub fn new(executor: Executor) -> Self {
        Reservations {
            nonces: Default::default(),
            executor,
        }
    }

    /// Reserves a nonce for `sender` that is at least `minimal`.
    pub fn reserve(&mut self, sender: Address, minimal: U256) -> Reserved {
        if self.nonces.len() + 1 > Self::CLEAN_AT {
            self.nonces.retain(|_, v| !v.is_empty());
        }
        let executor = &self.executor;
        self.nonces
            .entry(sender)
            .or_insert_with(move || SenderReservations::new(executor.clone()))
            .reserve_nonce(minimal)
    }
}

/// Manages currently reserved and prospective nonces.
#[derive(Debug)]
pub struct SenderReservations {
    /// Receiver of previous reservation's completed nonce (if any).
    previous: Option<oneshot::Receiver<U256>>,
    previous_ready: Arc<AtomicBool>,
    executor: Executor,
    prospective_value: U256,
    dropped: Arc<AtomicUsize>,
}

impl SenderReservations {
    pub fn new(executor: Executor) -> Self {
        SenderReservations {
            previous: None,
            previous_ready: Arc::new(AtomicBool::new(true)),
            executor,
            prospective_value: Default::default(),
            dropped: Default::default(),
        }
    }

    pub fn reserve_nonce(&mut self, minimal: U256) -> Reserved {
        let dropped = self.dropped.swap(0, atomic::Ordering::SeqCst);
        let prospective_value = cmp::max(
            minimal,
            self.prospective_value.saturating_sub(dropped.into()),
        );
        self.prospective_value = prospective_value + 1;

        let (next_tx, next_rx) = oneshot::channel::<U256>();
        let next_sent = Arc::new(AtomicBool::default());
        let executor = self.executor.clone();
        let dropped = self.dropped.clone();
        self.previous_ready = next_sent.clone();

        let previous_rx = self.previous.replace(next_rx);

        Reserved {
            previous_rx,
            immediate_value: minimal,
            next: Some(next_tx),
            next_sent,
            minimal,
            prospective_value,
            executor,
            dropped,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.previous_ready.load(atomic::Ordering::SeqCst)
    }
}

/// Represents a future nonce.
#[derive(Debug)]
pub struct Reserved {
    /// Receiver from previous reservation (None means first reservation).
    previous_rx: Option<oneshot::Receiver<U256>>,
    /// Immediate value when there's no previous (= minimal nonce).
    immediate_value: U256,
    next: Option<oneshot::Sender<U256>>,
    next_sent: Arc<AtomicBool>,
    minimal: U256,
    prospective_value: U256,
    executor: Executor,
    dropped: Arc<AtomicUsize>,
}

impl Reserved {
    pub fn prospective_value(&self) -> &U256 {
        &self.prospective_value
    }
}

impl std::future::Future for Reserved {
    type Output = Result<Ready, ()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let value = if let Some(ref mut rx) = self.previous_rx {
            match Pin::new(rx).poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Ok(v)) => v,
                Poll::Ready(Err(e)) => {
                    warn!("Unexpected nonce cancellation: {e}");
                    return Poll::Ready(Err(()));
                }
            }
        } else {
            self.immediate_value
        };

        let value = value.max(self.minimal);
        let matches_prospective = value == self.prospective_value;

        Poll::Ready(Ok(Ready {
            value,
            matches_prospective,
            next: self.next.take(),
            next_sent: self.next_sent.clone(),
            dropped: self.dropped.clone(),
        }))
    }
}

impl Drop for Reserved {
    fn drop(&mut self) {
        if let Some(next_tx) = self.next.take() {
            let next_sent = self.next_sent.clone();
            self.dropped.fetch_add(1, atomic::Ordering::SeqCst);
            let previous_rx = self.previous_rx.take();
            let immediate_value = self.immediate_value;
            let minimal = self.minimal;
            self.executor.spawn_03(async move {
                let value = if let Some(rx) = previous_rx {
                    rx.await.unwrap_or(immediate_value)
                } else {
                    immediate_value
                };
                let value = value.max(minimal);
                next_sent.store(true, atomic::Ordering::SeqCst);
                next_tx.send(value).ok();
            });
        }
    }
}

/// Represents a valid reserved nonce.
#[derive(Debug)]
pub struct Ready {
    value: U256,
    matches_prospective: bool,
    next: Option<oneshot::Sender<U256>>,
    next_sent: Arc<AtomicBool>,
    dropped: Arc<AtomicUsize>,
}

impl Ready {
    const RECV_PROOF: &'static str = "Receiver never dropped.";

    pub fn value(&self) -> &U256 {
        &self.value
    }

    pub fn matches_prospective(&self) -> bool {
        self.matches_prospective
    }

    pub fn mark_used(mut self) {
        let next = self
            .next
            .take()
            .expect("Nonce can be marked as used only once; qed");
        self.next_sent.store(true, atomic::Ordering::SeqCst);
        next.send(self.value + 1).expect(Self::RECV_PROOF);
    }
}

impl Drop for Ready {
    fn drop(&mut self) {
        if let Some(next) = self.next.take() {
            self.dropped.fetch_add(1, atomic::Ordering::SeqCst);
            self.next_sent.store(true, atomic::Ordering::SeqCst);
            next.send(self.value).expect(Self::RECV_PROOF);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_runtime::Runtime;

    fn block_on<F: std::future::Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(f)
    }

    #[test]
    fn should_reserve_a_set_of_nonces_and_resolve_them() {
        let runtime = Runtime::with_thread_per_future();
        let mut nonces = SenderReservations::new(runtime.executor());

        assert!(nonces.is_empty());
        let n1 = nonces.reserve_nonce(5.into());
        let n2 = nonces.reserve_nonce(5.into());
        let n3 = nonces.reserve_nonce(5.into());
        let n4 = nonces.reserve_nonce(5.into());
        assert!(!nonces.is_empty());

        let r = block_on(n1).unwrap();
        assert_eq!(r.value(), &U256::from(5));
        assert!(r.matches_prospective());
        r.mark_used();

        drop(n2);

        let r = block_on(n3).unwrap();
        drop(r);

        let r = block_on(n4).unwrap();
        assert_eq!(r.value(), &U256::from(6));
        assert!(!r.matches_prospective());
        r.mark_used();

        let n5 = nonces.reserve_nonce(5.into());
        let r = block_on(n5).unwrap();
        assert_eq!(r.value(), &U256::from(7));
        assert!(r.matches_prospective());
        r.mark_used();

        let n6 = nonces.reserve_nonce(10.into());
        let r = block_on(n6).unwrap();
        assert_eq!(r.value(), &U256::from(10));
        assert!(r.matches_prospective());
        r.mark_used();

        assert!(nonces.is_empty());
    }

    #[test]
    fn should_return_prospective_nonce() {
        let runtime = Runtime::with_thread_per_future();
        let mut nonces = SenderReservations::new(runtime.executor());

        let n1 = nonces.reserve_nonce(5.into());
        let n2 = nonces.reserve_nonce(5.into());

        assert_eq!(n1.prospective_value(), &U256::from(5));
        assert_eq!(n2.prospective_value(), &U256::from(6));
    }
}
