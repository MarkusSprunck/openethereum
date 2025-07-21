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

    /// Fetch with futures 0.1 compatibility
    fn fetch_compat(&self, request: Request, abort: Abort) -> FetchResult01;
}

impl ClientCompatExt for Client {
    fn get_compat(&self, url: &str, abort: Abort) -> FetchResult01 {
        Box::new(self.get(url, abort).compat())
    }

    fn fetch_compat(&self, request: Request, abort: Abort) -> FetchResult01 {
        Box::new(self.fetch(request, abort).compat())
    }
}
