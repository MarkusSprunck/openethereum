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

//! Generic poll manager for Pub-Sub.

use parking_lot::Mutex;
use std::{
    pin::Pin,
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
};

use futures::{
    channel::mpsc,
    future,
    FutureExt, SinkExt,
};
use jsonrpc_core::{
    self as core,
    MetaIoHandler,
};
use jsonrpc_pubsub::SubscriptionId;

use crate::v1::{helpers::Subscribers, metadata::Metadata};

#[derive(Debug)]
struct Subscription {
    metadata: Metadata,
    method: String,
    params: core::Params,
    sink: mpsc::Sender<Result<core::Value, core::Error>>,
    /// a flag if subscription is still active and last returned value
    last_result: Arc<(AtomicBool, Mutex<Option<core::Output>>)>,
}

/// A struct managing all subscriptions.
pub struct GenericPollManager<S: core::Middleware<Metadata>> {
    subscribers: Subscribers<Subscription>,
    rpc: MetaIoHandler<Metadata, S>,
}

impl<S: core::Middleware<Metadata>> GenericPollManager<S> {
    /// Creates new poll manager
    pub fn new(rpc: MetaIoHandler<Metadata, S>) -> Self {
        GenericPollManager {
            subscribers: Default::default(),
            rpc,
        }
    }

    /// Creates new poll manager with deterministic ids.
    #[cfg(test)]
    pub fn new_test(rpc: MetaIoHandler<Metadata, S>) -> Self {
        let mut manager = Self::new(rpc);
        manager.subscribers = Subscribers::default();
        manager
    }

    pub fn subscribe(
        &mut self,
        metadata: Metadata,
        method: String,
        params: core::Params,
    ) -> (
        SubscriptionId,
        mpsc::Receiver<Result<core::Value, core::Error>>,
    ) {
        let (sink, stream) = mpsc::channel(1);
        let subscription = Subscription {
            metadata,
            method,
            params,
            sink,
            last_result: Default::default(),
        };
        let id = self.subscribers.insert(subscription);
        (id, stream)
    }

    pub fn unsubscribe(&mut self, id: &SubscriptionId) -> bool {
        debug!(target: "pubsub", "Removing subscription: {id:?}");
        self.subscribers
            .remove(id)
            .map(|subscription| {
                subscription
                    .last_result
                    .0
                    .store(true, atomic::Ordering::SeqCst);
            })
            .is_some()
    }

    pub fn tick(&self) -> Pin<Box<dyn future::Future<Output = ()> + Send>> {
        let mut futures = Vec::new();
        for (id, subscription) in self.subscribers.iter() {
            let call = core::MethodCall {
                jsonrpc: Some(core::Version::V2),
                id: core::Id::Str(id.as_string()),
                method: subscription.method.clone(),
                params: subscription.params.clone(),
            };
            trace!(target: "pubsub", "Polling method: {call:?}");
            let result = self
                .rpc
                .handle_call(call.into(), subscription.metadata.clone());

            let last_result = subscription.last_result.clone();
            let mut sender = subscription.sink.clone();

            let result = result.then(move |response| async move {
                // quick check if the subscription is still valid
                if last_result.0.load(atomic::Ordering::SeqCst) {
                    return;
                }

                // Check and update last result (drop the guard before await)
                let should_send = {
                    let mut last = last_result.1.lock();
                    if *last != response && response.is_some() {
                        let output = response.as_ref().expect("Existence proved by the condition.");
                        debug!(target: "pubsub", "Got new response, sending: {output:?}");
                        *last = response.clone();
                        Some(match response.expect("checked above") {
                            core::Output::Success(core::Success { result, .. }) => Ok(result),
                            core::Output::Failure(core::Failure { error, .. }) => Err(error),
                        })
                    } else {
                        trace!(target: "pubsub", "Response was not changed: {response:?}");
                        None
                    }
                }; // MutexGuard dropped here, before any await

                if let Some(send) = should_send {
                    let _ = sender.send(send).await;
                }
            });

            futures.push(result);
        }
        Box::pin(future::join_all(futures).map(|_| ()))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{self, AtomicBool};

    use futures::StreamExt;
    use jsonrpc_core::{MetaIoHandler, NoopMiddleware, Params, Value};
    use jsonrpc_pubsub::SubscriptionId;

    use super::GenericPollManager;

    fn poll_manager() -> GenericPollManager<NoopMiddleware> {
        let mut io = MetaIoHandler::default();
        let called = AtomicBool::new(false);
        io.add_method("hello", move |_| {
            if called.load(atomic::Ordering::SeqCst) {
                futures::future::ready(Ok(Value::String("world".into())))
            } else {
                called.store(true, atomic::Ordering::SeqCst);
                futures::future::ready(Ok(Value::String("hello".into())))
            }
        });
        GenericPollManager::new_test(io)
    }

    #[test]
    fn should_poll_subscribed_method() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let mut poll_manager = poll_manager();
        let (id, mut rx) =
            poll_manager.subscribe(Default::default(), "hello".into(), Params::None);
        assert_eq!(id, SubscriptionId::String("0x43ca64edf03768e1".into()));

        rt.block_on(poll_manager.tick());
        let res = rt.block_on(rx.next());
        assert_eq!(res, Some(Ok(Value::String("hello".into()))));

        rt.block_on(poll_manager.tick());
        let res = rt.block_on(rx.next());
        assert_eq!(res, Some(Ok(Value::String("world".into()))));

        rt.block_on(poll_manager.tick());
        poll_manager.unsubscribe(&id);
        assert_eq!(rt.block_on(rx.next()), None);
    }
}
