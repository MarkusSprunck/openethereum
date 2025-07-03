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



use log::{debug, error, trace};

use bytes::Bytes;
use futures::{Future, Stream, FutureExt};
use futures::channel::oneshot;
use futures::task::{Poll, Context};
use std::pin::Pin;
use http::{HeaderMap, HeaderValue, StatusCode, Method};
use http::header::{self, IntoHeaderName};
use hyper::body::HttpBody;
use std::{cmp::min, fmt, io, thread, time::Duration};
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::RecvTimeoutError;
use tokio::sync::mpsc as tokio_mpsc;
use url::Url;

const MAX_SIZE: usize = 64 * 1024 * 1024;
const MAX_SECS: Duration = Duration::from_secs(5);
const MAX_REDR: usize = 5;

/// A handle to abort requests.
///
/// Requests are either aborted based on reaching thresholds such as
/// maximum response size, timeouts or too many redirects, or else
/// they can be aborted explicitly by the calling code.
#[derive(Clone, Debug)]
pub struct Abort {
    abort: Arc<AtomicBool>,
    size: usize,
    time: Duration,
    redir: usize,
}

impl Default for Abort {
    fn default() -> Abort {
        Abort {
            abort: Arc::new(AtomicBool::new(false)),
            size: MAX_SIZE,
            time: MAX_SECS,
            redir: MAX_REDR,
        }
    }
}

impl From<Arc<AtomicBool>> for Abort {
    fn from(a: Arc<AtomicBool>) -> Abort {
        Abort {
            abort: a,
            size: MAX_SIZE,
            time: MAX_SECS,
            redir: MAX_REDR,
        }
    }
}

impl Abort {
    /// True if `abort` has been invoked.
    pub fn is_aborted(&self) -> bool {
        self.abort.load(Ordering::SeqCst)
    }

    /// The maximum response body size.
    pub fn max_size(&self) -> usize {
        self.size
    }

    /// The maximum total time, including redirects.
    pub fn max_duration(&self) -> Duration {
        self.time
    }

    /// The maximum number of redirects to allow.
    pub fn max_redirects(&self) -> usize {
        self.redir
    }

    /// Mark as aborted.
    pub fn abort(&self) {
        self.abort.store(true, Ordering::SeqCst)
    }

    /// Set the maximum reponse body size.
    pub fn with_max_size(self, n: usize) -> Abort {
        Abort { size: n, ..self }
    }

    /// Set the maximum duration (including redirects).
    pub fn with_max_duration(self, d: Duration) -> Abort {
        Abort { time: d, ..self }
    }

    /// Set the maximum number of redirects to follow.
    pub fn with_max_redirects(self, n: usize) -> Abort {
        Abort { redir: n, ..self }
    }
}

/// Types which retrieve content from some URL.
pub trait Fetch: Clone + Send + Sync + 'static {
    /// The result future.
    type Result: Future<Output = Result<Response, Error>> + Send + 'static;

    /// Make a request to given URL
    fn fetch(&self, request: Request, abort: Abort) -> Self::Result;

    /// Get content from some URL.
    fn get(&self, url: &str, abort: Abort) -> Self::Result;

    /// Post content to some URL.
    fn post(&self, url: &str, abort: Abort) -> Self::Result;
}

type TxResponse = oneshot::Sender<Result<Response, Error>>;
type TxStartup = std::sync::mpsc::SyncSender<Result<(), std::io::Error>>;
type ChanItem = Option<(Request, Abort, TxResponse)>;

/// A handle to fetch client.
///
/// Implements `Fetch` trait to perform HTTP requests.
#[derive(Debug)]
pub struct Client {
    runtime: tokio_mpsc::Sender<ChanItem>,
    refs: Arc<AtomicUsize>,
}

// When cloning a client we increment the internal reference counter.
impl Clone for Client {
    fn clone(&self) -> Client {
        self.refs.fetch_add(1, Ordering::SeqCst);
        Client {
            runtime: self.runtime.clone(),
            refs: self.refs.clone(),
        }
    }
}

// When dropping a client, we decrement the reference counter.
// Once it reaches 0 we terminate the background thread.
impl Drop for Client {
    fn drop(&mut self) {
        if self.refs.fetch_sub(1, Ordering::SeqCst) == 1 {
            // ignore send error as it means the background thread is gone already
            let _ = self.runtime.try_send(None);
        }
    }
}

impl Client {
    /// Create a new fetch client.
    pub fn new() -> Result<Self, Error> {
        let (tx_start, rx_start) = std::sync::mpsc::sync_channel(1);
        let (tx_proto, rx_proto) = tokio_mpsc::channel(64);

        Client::background_thread(tx_start, rx_proto)?;

        match rx_start.recv_timeout(Duration::from_secs(10)) {
            Err(RecvTimeoutError::Timeout) => {
                error!(target: "fetch", "timeout starting background thread");
                return Err(Error::BackgroundThreadDead);
            }
            Err(RecvTimeoutError::Disconnected) => {
                error!(target: "fetch", "background thread gone");
                return Err(Error::BackgroundThreadDead);
            }
            Ok(Err(e)) => {
                error!(target: "fetch", "error starting background thread: {}", e);
                return Err(e.into());
            }
            Ok(Ok(())) => {}
        }

        Ok(Client {
            runtime: tx_proto,
            refs: Arc::new(AtomicUsize::new(1)),
        })
    }

    async fn execute_request_with_redirects(
        client: hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
        mut request: Request,
        abort: Abort,
    ) -> Result<Response, Error> {
        let mut redirects = 0;
        
        loop {
            if abort.is_aborted() {
                debug!(target: "fetch", "fetch of {} aborted", request.url());
                return Err(Error::Aborted);
            }
            
            let url = request.url().clone();
            let hyper_request = request.clone().into();
            
            match client.request(hyper_request).await {
                Ok(hyper_resp) => {
                    let resp = Response::new(url, hyper_resp, abort.clone());
                    
                    if abort.is_aborted() {
                        debug!(target: "fetch", "fetch of {} aborted", request.url());
                        return Err(Error::Aborted);
                    }
                    
                    if let Some((next_url, preserve_method)) = redirect_location(request.url().clone(), &resp) {
                        if redirects >= abort.max_redirects() {
                            return Err(Error::TooManyRedirects);
                        }
                        
                        request = if preserve_method {
                            let mut new_request = request.clone();
                            new_request.set_url(next_url);
                            new_request
                        } else {
                            Request::new(next_url, Method::GET)
                        };
                        
                        redirects += 1;
                        continue;
                    } else {
                        if let Some(ref h_val) = resp.headers.get(header::CONTENT_LENGTH) {
                            let content_len = h_val
                                .to_str()
                                .map_err(Error::HyperHeaderToStrError)?
                                .parse::<u64>()
                                .map_err(Error::ParseInt)?;
                            
                            if content_len > abort.max_size() as u64 {
                                return Err(Error::SizeLimit);
                            }
                        }
                        return Ok(resp);
                    }
                }
                Err(e) => return Err(Error::Hyper(e)),
            }
        }
    }

    fn background_thread(
        tx_start: TxStartup,
        mut rx_proto: tokio_mpsc::Receiver<ChanItem>,
    ) -> io::Result<thread::JoinHandle<()>> {
        thread::Builder::new().name("fetch".into()).spawn(move || {
            let runtime = match tokio::runtime::Runtime::new() {
                Ok(c) => c,
                Err(e) => return tx_start.send(Err(e)).unwrap_or(()),
            };

            let hyper = hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots());

            let future = async move {
                while let Some(item) = rx_proto.recv().await {
                    if item.is_none() {
                        break;
                    }
                    let (request, abort, sender) = item.unwrap();
                    
                    trace!(target: "fetch", "new request to {}", request.url());
                    if abort.is_aborted() {
                        sender.send(Err(Error::Aborted)).unwrap_or(());
                        continue;
                    }
                    let client = hyper.clone();
                    let fut = Self::execute_request_with_redirects(client, request, abort)
                        .then(move |result| {
                            sender.send(result).unwrap_or(());
                            futures::future::ready(())
                        });
                    tokio::spawn(fut);
                    trace!(target: "fetch", "waiting for next request ...");
                }
                Ok::<(), ()>(())
            };

            tx_start.send(Ok(())).unwrap_or(());

            debug!(target: "fetch", "processing requests ...");
            if let Err(()) = runtime.block_on(future) {
                error!(target: "fetch", "error while executing future")
            }
            debug!(target: "fetch", "fetch background thread finished")
        })
    }
}

impl Fetch for Client {
    type Result = std::pin::Pin<Box<dyn Future<Output = Result<Response, Error>> + Send + 'static>>;

    fn fetch(&self, request: Request, abort: Abort) -> Self::Result {
        debug!(target: "fetch", "fetching: {:?}", request.url());
        if abort.is_aborted() {
            return Box::pin(futures::future::ready(Err(Error::Aborted)));
        }
        let (tx_res, rx_res) = oneshot::channel();
        let maxdur = abort.max_duration();
        let sender = self.runtime.clone();
        
        let future = async move {
            sender
                .try_send(Some((request, abort, tx_res)))
                .map_err(|e| {
                    error!(target: "fetch", "failed to schedule request: {}", e);
                    Error::BackgroundThreadDead
                })?;
            
            let result = rx_res.await.map_err(|_| Error::BackgroundThreadDead)?;
            result
        };

        let timed_future = async move {
            match tokio::time::timeout(maxdur, future).await {
                Ok(result) => result,
                Err(_) => Err(Error::Timeout),
            }
        };

        Box::pin(timed_future)
    }

    /// Get content from some URL.
    fn get(&self, url: &str, abort: Abort) -> Self::Result {
        let url: Url = match url.parse() {
            Ok(u) => u,
            Err(e) => return Box::pin(futures::future::ready(Err(e.into()))),
        };
        self.fetch(Request::get(url), abort)
    }

    /// Post content to some URL.
    fn post(&self, url: &str, abort: Abort) -> Self::Result {
        let url: Url = match url.parse() {
            Ok(u) => u,
            Err(e) => return Box::pin(futures::future::ready(Err(e.into()))),
        };
        self.fetch(Request::post(url), abort)
    }
}

// Extract redirect location from response. The second return value indicate whether the original method should be preserved.
fn redirect_location(u: Url, r: &Response) -> Option<(Url, bool)> {
    let preserve_method = match r.status() {
        StatusCode::TEMPORARY_REDIRECT | StatusCode::PERMANENT_REDIRECT => true,
        _ => false,
    };
    match r.status() {
        StatusCode::MOVED_PERMANENTLY
        | StatusCode::PERMANENT_REDIRECT
        | StatusCode::TEMPORARY_REDIRECT
        | StatusCode::FOUND
        | StatusCode::SEE_OTHER => r.headers.get(header::LOCATION).and_then(|loc| {
            loc.to_str()
                .ok()
                .and_then(|loc_s| u.join(loc_s).ok().map(|url| (url, preserve_method)))
        }),
        _ => None,
    }
}

/// A wrapper for hyper::Request using Url and with methods.
#[derive(Debug, Clone)]
pub struct Request {
    url: Url,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
}

impl Request {
    /// Create a new request, with given url and method.
    pub fn new(url: Url, method: Method) -> Request {
        Request {
            url,
            method,
            headers: HeaderMap::new(),
            body: Default::default(),
        }
    }

    /// Create a new GET request.
    pub fn get(url: Url) -> Request {
        Request::new(url, Method::GET)
    }

    /// Create a new empty POST request.
    pub fn post(url: Url) -> Request {
        Request::new(url, Method::POST)
    }

    /// Read the url.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Read the request headers.
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get a mutable reference to the headers.
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    /// Set the body of the request.
    pub fn set_body<T: Into<Bytes>>(&mut self, body: T) {
        self.body = body.into();
    }

    /// Set the url of the request.
    pub fn set_url(&mut self, url: Url) {
        self.url = url;
    }

    /// Consume self, and return it with the added given header.
    pub fn with_header<K>(mut self, key: K, val: HeaderValue) -> Self
    where
        K: IntoHeaderName,
    {
        self.headers_mut().append(key, val);
        self
    }

    /// Consume self, and return it with the body.
    pub fn with_body<T: Into<Bytes>>(mut self, body: T) -> Self {
        self.body = body.into();
        self
    }
}

impl From<Request> for hyper::Request<hyper::Body> {
    fn from(req: Request) -> hyper::Request<hyper::Body> {
        let mut r = hyper::Request::builder()
            .method(req.method)
            .uri(req.url.as_str())
            .body(hyper::Body::from(req.body))
            .expect("Request conversion to hyper is infallible; qed");

        *r.headers_mut() = req.headers;
        r
    }
}

/// A wrapper for hyper::Response
#[derive(Debug)]
pub struct Response {
    url: Url,
    status: StatusCode,
    headers: HeaderMap,
    body: hyper::Body,
    abort: Abort,
    nread: usize,
}

impl Response {
    /// Create a new response, wrapping a hyper response.
    pub fn new(u: Url, r: hyper::Response<hyper::Body>, a: Abort) -> Response {
        Response {
            url: u,
            status: r.status(),
            headers: r.headers().clone(),
            body: r.into_body(),
            abort: a,
            nread: 0,
        }
    }

    /// The response status.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Status code == OK (200)?
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    /// Is the content-type text/html?
    pub fn is_html(&self) -> bool {
        self.headers
            .get(header::CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
            .map_or(false, |s| s.contains("text/html"))
    }
}

impl Stream for Response {
    type Item = Result<Bytes, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.abort.is_aborted() {
            debug!(target: "fetch", "fetch of {} aborted", self.url);
            return Poll::Ready(Some(Err(Error::Aborted)));
        }
        match Pin::new(&mut self.body).poll_data(cx) {
            Poll::Ready(Some(Ok(c))) => {
                if self.nread + c.len() > self.abort.max_size() {
                    debug!(target: "fetch", "size limit {:?} for {} exceeded", self.abort.max_size(), self.url);
                    return Poll::Ready(Some(Err(Error::SizeLimit)));
                }
                self.nread += c.len();
                Poll::Ready(Some(Ok(c)))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(Error::Hyper(e)))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// `BodyReader` serves as an adapter from async to sync I/O.
///
/// It implements `io::Read` by using `spawn_blocking` to safely bridge
/// async hyper Body to sync Read without causing deadlocks.
pub struct BodyReader {
    chunk: bytes::Bytes,
    body: Option<hyper::Body>,
    abort: Abort,
    offset: usize,
    count: usize,
}

impl BodyReader {
    /// Create a new body reader for the given response.
    pub fn new(r: Response) -> BodyReader {
        BodyReader {
            body: Some(r.body),
            chunk: Default::default(),
            abort: r.abort,
            offset: 0,
            count: 0,
        }
    }
}

impl io::Read for BodyReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut n = 0;
        
        while self.body.is_some() {
            // Can we still read from the current chunk?
            if self.offset < self.chunk.len() {
                let k = min(self.chunk.len() - self.offset, buf.len() - n);
                if self.count + k > self.abort.max_size() {
                    debug!(target: "fetch", "size limit {:?} exceeded", self.abort.max_size());
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "size limit exceeded",
                    ));
                }
                let c = &self.chunk[self.offset..self.offset + k];
                (&mut buf[n..n + k]).copy_from_slice(c);
                self.offset += k;
                self.count += k;
                n += k;
                if n == buf.len() {
                    break;
                }
            } else {
                // Need to get the next chunk from the async body
                let mut body = self
                    .body
                    .take()
                    .expect("loop condition ensures `self.body` is always defined; qed");
                
                // Use spawn_blocking to safely bridge async/sync
                let result = std::thread::scope(|s| {
                    let handle = s.spawn(|| {
                        // Create a new runtime for this blocking operation
                        let rt = tokio::runtime::Runtime::new().map_err(|e| {
                            io::Error::new(io::ErrorKind::Other, format!("runtime creation failed: {}", e))
                        })?;
                        
                        rt.block_on(async {
                            use hyper::body::HttpBody;
                            match body.data().await {
                                Some(Ok(chunk)) => Ok(Some(chunk)),
                                Some(Err(e)) => Err(io::Error::new(
                                    io::ErrorKind::Other, 
                                    format!("body read error: {}", e)
                                )),
                                None => Ok(None),
                            }
                        })
                    });
                    
                    handle.join().map_err(|_| {
                        io::Error::new(io::ErrorKind::Other, "thread join failed")
                    })?
                });
                
                match result {
                    Ok(Some(chunk)) => {
                        self.body = Some(body);
                        self.chunk = chunk;
                        self.offset = 0;
                    }
                    Ok(None) => break, // body is exhausted
                    Err(e) => {
                        error!(target: "fetch", "failed to read chunk: {}", e);
                        return Err(e);
                    }
                }
            }
        }
        Ok(n)
    }
}

/// Fetch error cases.
#[derive(Debug)]
pub enum Error {
    /// Hyper gave us an error.
    Hyper(hyper::Error),
    /// A hyper header conversion error.
    HyperHeaderToStrError(hyper::header::ToStrError),
    /// An integer parsing error.
    ParseInt(std::num::ParseIntError),
    /// Some I/O error occured.
    Io(io::Error),
    /// Invalid URLs where attempted to parse.
    Url(url::ParseError),
    /// Calling code invoked `Abort::abort`.
    Aborted,
    /// Too many redirects have been encountered.
    TooManyRedirects,
    /// tokio-timer inner future gave us an error.
    TokioTimeoutInnerVal(String),
    /// tokio-time gave us an error.
    TokioTime(Option<tokio::time::error::Elapsed>),
    /// The maximum duration was reached.
    Timeout,
    /// The response body is too large.
    SizeLimit,
    /// The background processing thread does not run.
    BackgroundThreadDead,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Aborted => write!(fmt, "The request has been aborted."),
            Error::Hyper(ref e) => write!(fmt, "{}", e),
            Error::HyperHeaderToStrError(ref e) => write!(fmt, "{}", e),
            Error::ParseInt(ref e) => write!(fmt, "{}", e),
            Error::Url(ref e) => write!(fmt, "{}", e),
            Error::Io(ref e) => write!(fmt, "{}", e),
            Error::BackgroundThreadDead => write!(fmt, "background thread gond"),
            Error::TooManyRedirects => write!(fmt, "too many redirects"),
            Error::TokioTimeoutInnerVal(ref s) => {
                write!(fmt, "tokio timer inner value error: {:?}", s)
            }
            Error::TokioTime(ref e) => write!(fmt, "tokio timer error: {:?}", e),
            Error::Timeout => write!(fmt, "request timed out"),
            Error::SizeLimit => write!(fmt, "size limit reached"),
        }
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        "Fetch client error"
    }
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Self {
        Error::Hyper(e)
    }
}

impl From<hyper::header::ToStrError> for Error {
    fn from(e: hyper::header::ToStrError) -> Self {
        Error::HyperHeaderToStrError(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Error::ParseInt(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Error::Url(e)
    }
}

impl From<tokio::time::error::Elapsed> for Error {
    fn from(_e: tokio::time::error::Elapsed) -> Self {
        Error::Timeout
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::{channel::oneshot, StreamExt};
    use hyper::{server::conn::AddrStream, service::{make_service_fn, Service}, Body, Request, Response as HyperResponse, Server, StatusCode};
    use std::{convert::Infallible, net::SocketAddr};
    use tokio::runtime::Runtime;

    const ADDRESS: &str = "127.0.0.1:0";

    #[test]
    fn it_should_fetch() {
        let server = TestServer::run();
        let client = Client::new().unwrap();
        let runtime = Runtime::new().unwrap();

        runtime.block_on(async {
            let resp = client
                .get(&format!("http://{}?123", server.addr()), Abort::default())
                .await
                .expect("Request failed");
            
            assert!(resp.is_success());
            
            let mut body = Vec::new();
            let mut resp_stream = resp;
            while let Some(chunk) = resp_stream.next().await {
                let chunk = chunk.expect("Failed to read chunk");
                body.extend_from_slice(&chunk);
            }
            assert_eq!(&body[..], b"123");
        });
    }

    #[test]
    fn it_should_timeout() {
        let server = TestServer::run();
        let client = Client::new().unwrap();
        let runtime = Runtime::new().unwrap();

        let abort = Abort::default().with_max_duration(Duration::from_secs(1));

        runtime.block_on(async {
            match client.get(&format!("http://{}/delay?3", server.addr()), abort).await {
                Err(Error::Timeout) => {},
                other => panic!("expected timeout, got {:?}", other),
            }
        });
    }

    #[test]
    fn it_should_follow_redirects() {
        let server = TestServer::run();
        let client = Client::new().unwrap();
        let runtime = Runtime::new().unwrap();

        let abort = Abort::default();

        runtime.block_on(async {
            let resp = client
                .get(
                    &format!(
                        "http://{}/redirect?http://{}/",
                        server.addr(),
                        server.addr()
                    ),
                    abort,
                )
                .await
                .expect("Request failed");
            
            assert!(resp.is_success(), "Response unsuccessful");
        });
    }

    #[test]
    fn it_should_follow_relative_redirects() {
        let server = TestServer::run();
        let client = Client::new().unwrap();
        let runtime = Runtime::new().unwrap();

        let abort = Abort::default().with_max_redirects(4);
        
        runtime.block_on(async {
            let resp = client
                .get(&format!("http://{}/redirect?/", server.addr()), abort)
                .await
                .expect("Request failed");
            
            assert!(resp.is_success(), "Response unsuccessful");
        });
    }

    #[test]
    fn it_should_not_follow_too_many_redirects() {
        let server = TestServer::run();
        let client = Client::new().unwrap();
        let runtime = Runtime::new().unwrap();

        let abort = Abort::default().with_max_redirects(3);
        
        runtime.block_on(async {
            match client.get(&format!("http://{}/loop", server.addr()), abort).await {
                Err(Error::TooManyRedirects) => {},
                other => panic!("expected too many redirects error, got {:?}", other),
            }
        });
    }

    #[test]
    fn it_should_read_data() {
        let server = TestServer::run();
        let client = Client::new().unwrap();
        let runtime = Runtime::new().unwrap();

        let abort = Abort::default();
        
        runtime.block_on(async {
            let resp = client
                .get(
                    &format!("http://{}?abcdefghijklmnopqrstuvwxyz", server.addr()),
                    abort,
                )
                .await
                .expect("Request failed");
            
            assert!(resp.is_success(), "Response unsuccessful");
            
            let mut body = Vec::new();
            let mut resp_stream = resp;
            while let Some(chunk) = resp_stream.next().await {
                let chunk = chunk.expect("Failed to read chunk");
                body.extend_from_slice(&chunk);
            }
            assert_eq!(&body[..], b"abcdefghijklmnopqrstuvwxyz");
        });
    }

    #[test]
    fn it_should_not_read_too_much_data() {
        let server = TestServer::run();
        let client = Client::new().unwrap();
        let runtime = Runtime::new().unwrap();

        let abort = Abort::default().with_max_size(3);
        
        runtime.block_on(async {
            match client.get(&format!("http://{}/?1234", server.addr()), abort).await {
                Err(Error::SizeLimit) => {},
                Ok(resp) => {
                    assert!(resp.is_success(), "Response unsuccessful");
                    
                    let mut body = Vec::new();
                    let mut resp_stream = resp;
                    let mut result = Ok(());
                    
                    while let Some(chunk) = resp_stream.next().await {
                        match chunk {
                            Ok(chunk_data) => body.extend_from_slice(&chunk_data),
                            Err(Error::SizeLimit) => {
                                result = Err(Error::SizeLimit);
                                break;
                            }
                            Err(e) => panic!("unexpected error: {:?}", e),
                        }
                    }
                    
                    match result {
                        Err(Error::SizeLimit) => {},
                        _ => panic!("expected size limit error"),
                    }
                }
                other => panic!("Expected `Error::SizeLimit`, got: {:?}", other),
            }
        });
    }

    #[test]
    fn it_should_not_read_too_much_data_sync() {
        let server = TestServer::run();
        let client = Client::new().unwrap();
        let runtime = Runtime::new().unwrap();

        // let abort = Abort::default().with_max_size(3);
        // let resp = client.get(&format!("http://{}/?1234", server.addr()), abort).wait().unwrap();
        // assert!(resp.is_success());
        // let mut buffer = Vec::new();
        // let mut reader = BodyReader::new(resp);
        // match reader.read_to_end(&mut buffer) {
        // 	Err(ref e) if e.kind() == io::ErrorKind::PermissionDenied => {}
        // 	other => panic!("expected size limit error, got {:?}", other)
        // }

        // FIXME (c0gent): The prior version of this test (pre-hyper-0.12,
        // commented out above) is not possible to recreate. It relied on an
        // apparent bug in `Client::background_thread` which suppressed the
        // `SizeLimit` error from occurring. This is due to the headers
        // collection not returning a value for content length when queried.
        // The precise reason why this was happening is unclear.

        let abort = Abort::default().with_max_size(3);
        
        runtime.block_on(async {
            match client.get(&format!("http://{}/?1234", server.addr()), abort).await {
                Err(Error::SizeLimit) => {},
                Ok(resp) => {
                    assert!(resp.is_success());
                    
                    let mut body = Vec::new();
                    let mut resp_stream = resp;
                    let mut result = Ok(());
                    
                    while let Some(chunk) = resp_stream.next().await {
                        match chunk {
                            Ok(chunk_data) => body.extend_from_slice(&chunk_data),
                            Err(Error::SizeLimit) => {
                                result = Err(Error::SizeLimit);
                                break;
                            }
                            Err(e) => panic!("unexpected error: {:?}", e),
                        }
                    }
                    
                    match result {
                        Err(Error::SizeLimit) => {},
                        _ => panic!("expected size limit error"),
                    }
                }
                other => panic!("Expected `Error::SizeLimit`, got: {:?}", other),
            }
        });
    }

    struct TestServer;

    impl Service<Request<Body>> for TestServer {
        type Response = HyperResponse<Body>;
        type Error = Infallible;
        type Future = std::pin::Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

        fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: Request<Body>) -> Self::Future {
            let path = req.uri().path().to_string();
            let query = req.uri().query().unwrap_or("").to_string();
            
            Box::pin(async move {
                match path.as_str() {
                    "/" => {
                        let res = HyperResponse::new(Body::from(query));
                        Ok(res)
                    }
                    "/redirect" => {
                        let loc = if query.is_empty() { "/" } else { &query };
                        let res = HyperResponse::builder()
                            .status(StatusCode::MOVED_PERMANENTLY)
                            .header(hyper::header::LOCATION, loc)
                            .body(Body::empty())
                            .expect("Unable to create response");
                        Ok(res)
                    }
                    "/loop" => {
                        let res = HyperResponse::builder()
                            .status(StatusCode::MOVED_PERMANENTLY)
                            .header(hyper::header::LOCATION, "/loop")
                            .body(Body::empty())
                            .expect("Unable to create response");
                        Ok(res)
                    }
                    "/delay" => {
                        let dur = Duration::from_secs(query.parse().unwrap_or(0));
                        tokio::time::sleep(dur).await;
                        let res = HyperResponse::new(Body::empty());
                        Ok(res)
                    }
                    _ => {
                        let res = HyperResponse::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Body::empty())
                            .expect("Unable to create response");
                        Ok(res)
                    }
                }
            })
        }
    }

    impl TestServer {
        fn run() -> Handle {
            let (tx_start, rx_start) = std::sync::mpsc::sync_channel(1);
            let (tx_end, rx_end) = oneshot::channel();
            
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let addr: SocketAddr = ADDRESS.parse().unwrap();

                    let make_svc = make_service_fn(|_conn: &AddrStream| {
                        async { Ok::<_, Infallible>(TestServer) }
                    });

                    let server = Server::bind(&addr).serve(make_svc);
                    let actual_addr = server.local_addr();

                    tx_start.send(actual_addr).unwrap_or(());

                    let graceful = server.with_graceful_shutdown(async {
                        rx_end.await.ok();
                    });

                    if let Err(e) = graceful.await {
                        eprintln!("server error: {}", e);
                    }
                });
            });

            Handle(rx_start.recv().unwrap(), Some(tx_end))
        }
    }

    struct Handle(SocketAddr, Option<oneshot::Sender<()>>);

    impl Handle {
        fn addr(&self) -> SocketAddr {
            self.0
        }
    }

    impl Drop for Handle {
        fn drop(&mut self) {
            self.1.take().unwrap().send(()).unwrap();
        }
    }
}
