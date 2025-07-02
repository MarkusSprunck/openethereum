use crate::{Abort, Client, Error, Fetch, Request, Response};
use futures::TryFutureExt;
use futures_01::Future as Future01;

/// Type alias for futures 0.1 fetch result.
/// 
/// This represents a boxed future that is compatible with the futures 0.1 ecosystem,
/// returning a `Response` on success or an `Error` on failure.
pub type FetchResult01 = Box<dyn Future01<Item = Response, Error = Error> + Send>;

/// Extension trait to add compat methods to the regular Client
pub trait ClientCompatExt {
    /// Get content with futures 0.1 compatibility
    fn get_compat(&self, url: &str, abort: Abort) -> FetchResult01;
    
    /// Post content with futures 0.1 compatibility  
    fn post_compat(&self, url: &str, abort: Abort) -> FetchResult01;
    
    /// Fetch with futures 0.1 compatibility
    fn fetch_compat(&self, request: Request, abort: Abort) -> FetchResult01;
}

impl ClientCompatExt for Client {
    fn get_compat(&self, url: &str, abort: Abort) -> FetchResult01 {
        Box::new(self.get(url, abort).compat())
    }

    fn post_compat(&self, url: &str, abort: Abort) -> FetchResult01 {
        Box::new(self.post(url, abort).compat())
    }

    fn fetch_compat(&self, request: Request, abort: Abort) -> FetchResult01 {
        Box::new(self.fetch(request, abort).compat())
    }
}

/// A dedicated wrapper client for futures 0.1 compatibility
#[derive(Clone)]
pub struct ClientCompat {
    inner: Client,
}

impl ClientCompat {
    /// Create a new compat client wrapper
    pub fn new(num_dns_threads: usize) -> Result<Self, Error> {
        Ok(ClientCompat {
            inner: Client::new(num_dns_threads)?,
        })
    }

    /// Get content with futures 0.1 compatibility
    pub fn get(&self, url: &str, abort: Abort) -> FetchResult01 {
        Box::new(self.inner.get(url, abort).compat())
    }

    /// Post content with futures 0.1 compatibility  
    pub fn post(&self, url: &str, abort: Abort) -> FetchResult01 {
        Box::new(self.inner.post(url, abort).compat())
    }

    /// Fetch with futures 0.1 compatibility
    pub fn fetch(&self, request: Request, abort: Abort) -> FetchResult01 {
        Box::new(self.inner.fetch(request, abort).compat())
    }

    /// Get the inner client
    pub fn inner(&self) -> &Client {
        &self.inner
    }
} 