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
    futures::{future, Future, Sink, Stream},
    MetaIoHandler, Result,
};
use jsonrpc_pubsub::{typed::Subscriber, SubscriptionId};
use tokio_timer;

use parity_runtime::Executor;
use v1::{helpers::GenericPollManager, metadata::Metadata, traits::PubSub};

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

        let timer = tokio_timer::wheel()
            .tick_duration(Duration::from_millis(500))
            .build();

        // Start ticking
        let interval = timer.interval(Duration::from_millis(1000));
        executor.spawn(
            interval
                .map_err(|e| warn!("Polling timer error: {e:?}"))
                .for_each(move |()| {
                    if let Some(pm2) = pm2.upgrade() {
                        pm2.read().tick()
                    } else {
                        Box::new(future::err(()))
                    }
                }),
        );

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

impl<S: core::Middleware<Metadata>> PubSub for PubSubClient<S> {
    type Metadata = Metadata;

    fn parity_subscribe(
        &self,
        mut meta: Metadata,
        subscriber: Subscriber<core::Value>,
        method: String,
        params: Option<core::Params>,
    ) {
        let params = params.unwrap_or_else(|| core::Params::Array(vec![]));
        // Make sure to get rid of PubSub session otherwise it will never be dropped.
        meta.session = None;

        let mut poll_manager = self.poll_manager.write();
        let (id, receiver) = poll_manager.subscribe(meta, method, params);
        match subscriber.assign_id(id.clone()) {
            Ok(sink) => {
                self.executor.spawn(
                    receiver
                        .forward(sink.sink_map_err(|e| {
                            warn!("Cannot send notification: {e:?}");
                        }))
                        .map(|_| ()),
                );
            }
            Err(()) => {
                poll_manager.unsubscribe(&id);
            }
        }
    }

    fn parity_unsubscribe(&self, _: Option<Self::Metadata>, id: SubscriptionId) -> Result<bool> {
        let res = self.poll_manager.write().unsubscribe(&id);
        Ok(res)
    }
}
