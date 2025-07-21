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

extern crate fetch;
extern crate futures;
extern crate hyper;

use fetch::{ClientCompatExt, Fetch, FetchResult01, Request, Url};
use futures::{future, Future, TryFutureExt};
use http::StatusCode;
use hyper::Body;
use std::pin::Pin;

#[derive(Clone, Default)]
pub struct FakeFetch<T>
where
    T: Clone + Send + Sync,
{
    val: Option<T>,
}

impl<T> FakeFetch<T>
where
    T: Clone + Send + Sync,
{
    pub fn new(t: Option<T>) -> Self {
        FakeFetch { val: t }
    }
}

impl<T: 'static> Fetch for FakeFetch<T>
where
    T: Clone + Send + Sync,
{
    type Result =
        Pin<Box<dyn Future<Output = Result<fetch::Response, fetch::Error>> + Send + 'static>>;

    fn fetch(&self, request: Request, abort: fetch::Abort) -> Self::Result {
        let u = request.url().clone();
        Box::pin(future::ready(if self.val.is_some() {
            let r = hyper::Response::new("Some content".into());
            Ok(fetch::client::Response::new(u, r, abort))
        } else {
            let r = hyper::Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .expect("Nothing to parse, can not fail; qed");
            Ok(fetch::client::Response::new(u, r, abort))
        }))
    }

    fn get(&self, url: &str, abort: fetch::Abort) -> Self::Result {
        let url: Url = match url.parse() {
            Ok(u) => u,
            Err(e) => return Box::pin(future::ready(Err(e.into()))),
        };
        self.fetch(Request::get(url), abort)
    }

    fn post(&self, url: &str, abort: fetch::Abort) -> Self::Result {
        let url: Url = match url.parse() {
            Ok(u) => u,
            Err(e) => return Box::pin(future::ready(Err(e.into()))),
        };
        self.fetch(Request::post(url), abort)
    }
}

impl<T: 'static> ClientCompatExt for FakeFetch<T>
where
    T: Clone + Send + Sync,
{
    fn get_compat(&self, url: &str, abort: fetch::Abort) -> FetchResult01 {
        Box::new(self.get(url, abort).compat())
    }

    fn fetch_compat(&self, request: Request, abort: fetch::Abort) -> FetchResult01 {
        Box::new(self.fetch(request, abort).compat())
    }
}
