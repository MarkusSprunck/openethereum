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
    pin::Pin,
    task::{Context, Poll},
};

use futures::channel::oneshot;
use jsonrpc_core::Error;
use crate::v1::helpers::errors;

pub type Res<T> = Result<T, Error>;

pub struct Sender<T> {
    sender: oneshot::Sender<Res<T>>,
}

impl<T> Sender<T> {
    pub fn send(self, data: Res<T>) {
        let res = self.sender.send(data);
        if res.is_err() {
            debug!(target: "rpc", "Responding to a no longer active request.");
        }
    }
}

pub struct Receiver<T> {
    receiver: oneshot::Receiver<Res<T>>,
}

impl<T: Send> std::future::Future for Receiver<T> {
    type Output = Res<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.receiver).poll(cx) {
            Poll::Ready(Ok(Ok(res))) => Poll::Ready(Ok(res)),
            Poll::Ready(Ok(Err(err))) => Poll::Ready(Err(err)),
            Poll::Ready(Err(e)) => {
                debug!(target: "rpc", "Responding to a canceled request: {e:?}");
                Poll::Ready(Err(errors::internal("Request was canceled by client.", e)))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

pub fn oneshot<T>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = oneshot::channel();
    (Sender { sender: tx }, Receiver { receiver: rx })
}
