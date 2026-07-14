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

//! OpenEthereum-specific PUB-SUB rpc implementation.

use parking_lot::RwLock;
use std::{sync::Arc, time::Duration};

use jsonrpc_core::{
    self as core,
    MetaIoHandler, Result,
};
use jsonrpc_pubsub::{typed::Subscriber, SubscriptionId};

use parity_runtime::Executor;
use crate::v1::{helpers::GenericPollManager, metadata::Metadata, traits::PubSub};

/// Parity `PubSub` implementation.
pub struct PubSubClient<S: core::Middleware<Metadata>> {
    poll_manager: Arc<RwLock<GenericPollManager<S>>>,
    executor: Executor,
}

impl<S: core::Middleware<Metadata>> PubSubClient<S> {
    /// Creates new `PubSubClient`.
    pub fn new(rpc: MetaIoHandler<Metadata, S>, executor: Executor) -> Self {
        let poll_manager = Arc::new(RwLock::new(GenericPollManager::new(rpc)));
        let pm2 = Arc::downgrade(&poll_manager);

        // Start ticking every 1000ms using tokio::time::interval
        executor.spawn_03(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(1000));
            loop {
                interval.tick().await;
                if let Some(pm) = pm2.upgrade() {
                    // Get the tick future while holding the lock, then drop the lock before awaiting
                    let tick_fut = pm.read().tick();
                    tick_fut.await;
                } else {
                    break;
                }
            }
        });

        PubSubClient {
            poll_manager,
            executor,
        }
    }
}

impl PubSubClient<core::NoopMiddleware> {
    /// Creates new `PubSubClient` with deterministic ids.
    #[cfg(test)]
    #[must_use]
    pub fn new_test(
        rpc: MetaIoHandler<Metadata, core::NoopMiddleware>,
        executor: Executor,
    ) -> Self {
        let client = Self::new(MetaIoHandler::with_middleware(Default::default()), executor);
        *client.poll_manager.write() = GenericPollManager::new_test(rpc);
        client
    }
}

impl<S: core::Middleware<Metadata> + Unpin + 'static> PubSub for PubSubClient<S> {
    type Metadata = Metadata;

    fn parity_subscribe(
        &self,
        _meta: Metadata,
        subscriber: Subscriber<core::Value>,
        method: String,
        params: Option<core::Params>,
    ) {
        let params = params.unwrap_or(core::Params::None);
        let (id, mut stream) = {
            let mut manager = self.poll_manager.write();
            manager.subscribe(Default::default(), method, params)
        };

        // Assign the subscription ID to complete the jsonrpc-pubsub handshake.
        // If the session is already closed, assign_id returns Err(()), so we just return.
        if let Ok(sink) = subscriber.assign_id(id) {
            // Spawn a task that forwards notifications from the poll-manager's stream
            // to the subscriber's typed sink.
            self.executor.spawn_03(async move {
                use futures::StreamExt;
                while let Some(item) = stream.next().await {
                    if sink.notify(item).is_err() {
                        // Subscriber was dropped / session closed
                        break;
                    }
                }
            });
        }
    }

    fn parity_unsubscribe(&self, _meta: Option<Self::Metadata>, id: SubscriptionId) -> Result<bool> {
        let mut manager = self.poll_manager.write();
        Ok(manager.unsubscribe(&id))
    }
}
