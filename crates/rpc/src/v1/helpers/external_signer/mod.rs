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

//! An list of requests to be confirmed or signed by an external approver/signer.

use std::{ops::Deref, sync::Arc};

mod oneshot;
mod signing_queue;

#[cfg(test)]
pub use self::signing_queue::QueueEvent;
pub use self::signing_queue::{
    ConfirmationReceiver, ConfirmationResult, ConfirmationsQueue, SigningQueue,
};

/// Manages communication with Signer crate
pub struct SignerService {
    is_enabled: bool,
    queue: Arc<ConfirmationsQueue>,
    generate_new_token: Box<dyn Fn() -> Result<String, String> + Send + Sync + 'static>,
}

impl SignerService {
    /// Creates new Signer Service given function to generate new tokens.
    pub fn new<F>(new_token: F, is_enabled: bool) -> Self
    where
        F: Fn() -> Result<String, String> + Send + Sync + 'static,
    {
        SignerService {
            queue: Arc::new(ConfirmationsQueue::default()),
            generate_new_token: Box::new(new_token),
            is_enabled,
        }
    }

    /// Generates new signer authorization token.
    pub fn generate_token(&self) -> Result<String, String> {
        (self.generate_new_token)()
    }

    /// Returns a reference to `ConfirmationsQueue`
    #[must_use]
    pub fn queue(&self) -> Arc<ConfirmationsQueue> {
        self.queue.clone()
    }

    /// Returns true if Signer is enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    #[cfg(test)]
    /// Creates new Signer Service for tests.
    #[must_use]
    pub fn new_test(is_enabled: bool) -> Self {
        SignerService::new(|| Ok("new_token".into()), is_enabled)
    }
}

impl Deref for SignerService {
    type Target = ConfirmationsQueue;
    fn deref(&self) -> &Self::Target {
        &self.queue
    }
}
