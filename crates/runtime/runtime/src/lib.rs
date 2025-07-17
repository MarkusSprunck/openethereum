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

//! Tokio Runtime wrapper.

pub extern crate futures;
pub extern crate tokio;

// Re-export futures01 for backward compatibility
pub use futures01;

use futures::Future;
use std::{
    fmt,
    sync::mpsc,
    thread,
};
pub use tokio::{
    runtime::{Builder as TokioRuntimeBuilder, Handle as TokioHandle, Runtime as TokioRuntime},
    time::{sleep as delay, Sleep as Delay},
};

// Compatibility re-exports for users expecting old APIs
pub type TaskExecutor = TokioHandle;

/// Create a new runtime for each sync operation - avoids shared bottleneck and deadlocks
fn create_sync_runtime() -> TokioRuntime {
    TokioRuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create sync runtime")
}

/// Shared runtime for ThreadPerFuture mode - safe because each operation gets its own thread
fn get_thread_per_future_runtime() -> &'static TokioRuntime {
    static THREAD_PER_FUTURE_RT: std::sync::OnceLock<TokioRuntime> = std::sync::OnceLock::new();
    THREAD_PER_FUTURE_RT.get_or_init(|| {
        TokioRuntimeBuilder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create thread-per-future runtime")
    })
}
/// Runtime for futures.
///
/// Runs in a separate thread.
pub struct Runtime {
    executor: Executor,
    handle: RuntimeHandle,
}

impl Runtime {
    fn new(runtime_bldr: &mut TokioRuntimeBuilder) -> Self {
        let runtime = runtime_bldr.build().expect(
            "Building a Tokio runtime will only fail when mio components \
				cannot be initialized (catastrophic)",
        );
        let handle = runtime.handle().clone();
        let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
        let (tx, rx) = mpsc::channel();
        let thread_handle = thread::spawn(move || {
            tx.send(handle.clone())
                .expect("Rx is blocking upper thread.");

            // Keep the runtime alive and use it to block on the stop signal
            runtime.block_on(async {
                let _ = stop_rx.await;
            });
        });

        let executor = rx
            .recv()
            .expect("tx is transfered to a newly spawned thread.");

        Runtime {
            executor: Executor {
                inner: Mode::Tokio(executor),
            },
            handle: RuntimeHandle {
                close: Some(stop_tx),
                handle: Some(thread_handle),
            },
        }
    }

    /// Spawns a new tokio runtime with a default thread count on a background
    /// thread and returns a `Runtime` which can be used to spawn tasks via
    /// its executor.
    pub fn with_default_thread_count() -> Self {
        let mut runtime_bldr = TokioRuntimeBuilder::new_multi_thread();
        Self::new(&mut runtime_bldr)
    }

    /// Creates a single-threaded runtime using the production Self::new() code path.
    /// This creates an actual tokio current_thread runtime but should only be used
    /// by tests that don't need to make blocking .wait() calls (which can deadlock).
    pub fn with_single_thread() -> Self {
        let mut runtime_bldr = TokioRuntimeBuilder::new_current_thread();
        Self::new(&mut runtime_bldr)
    }

    /// Creates a runtime that spawns a new thread for each future execution.
    /// This mode is designed for tests that need to make blocking .wait() calls,
    /// as it avoids deadlocks by running each future in its own thread.
    pub fn with_thread_per_future() -> Self {
        Runtime {
            executor: Executor {
                inner: Mode::ThreadPerFuture,
            },
            handle: RuntimeHandle {
                close: None,
                handle: None,
            },
        }
    }

    /// Returns this runtime raw executor.
    ///
    /// Deprecated: Exists only to connect with current JSONRPC implementation.
    pub fn raw_executor(&self) -> TaskExecutor {
        if let Mode::Tokio(ref executor) = self.executor.inner {
            executor.clone()
        } else {
            panic!("Runtime is not initialized in Tokio mode.")
        }
    }

    /// Returns runtime executor.
    pub fn executor(&self) -> Executor {
        self.executor.clone()
    }
}

#[derive(Clone)]
enum Mode {
    Tokio(TokioHandle),
    Sync,
    ThreadPerFuture,
}

impl fmt::Debug for Mode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use self::Mode::*;

        match *self {
            Tokio(_) => write!(fmt, "tokio"),
            Sync => write!(fmt, "synchronous"),
            ThreadPerFuture => write!(fmt, "thread per future"),
        }
    }
}


#[derive(Debug, Clone)]
pub struct Executor {
    inner: Mode,
}

impl Executor {
    /// Executor for existing runtime.
    ///
    /// Deprecated: Exists only to connect with current JSONRPC implementation.
    pub fn new(executor: TaskExecutor) -> Self {
        Executor {
            inner: Mode::Tokio(executor),
        }
    }

    /// Synchronous executor, used mostly for tests.
    pub fn new_sync() -> Self {
        Executor { inner: Mode::Sync }
    }

    /// Spawns a new thread for each future (use only for tests).
    pub fn new_thread_per_future() -> Self {
        Executor {
            inner: Mode::ThreadPerFuture,
        }
    }

    /// Spawn a futures 0.1 future to this runtime (default method for backward compatibility)
    pub fn spawn<R>(&self, r: R)
    where
        R: futures01::IntoFuture<Item = (), Error = ()> + Send + 'static,
        R::Future: Send + 'static,
    {
        self.spawn_01(r);
    }

    /// Spawn a futures 0.3 future to this runtime
    pub fn spawn_03<R>(&self, r: R)
    where
        R: Future<Output = ()> + Send + 'static,
    {
        match self.inner {
            Mode::Tokio(ref executor) => {
                executor.spawn(r);
            }
            Mode::Sync => {
                create_sync_runtime().block_on(r);
            }
            Mode::ThreadPerFuture => {
                thread::spawn(move || {
                    get_thread_per_future_runtime().block_on(r);
                });
            }
        }
    }
}

// Compatibility layer for futures 0.1 users
impl Executor {
    /// Spawn a futures 0.1 future (for backward compatibility)
    pub fn spawn_01<R>(&self, r: R)
    where
        R: futures01::IntoFuture<Item = (), Error = ()> + Send + 'static,
        R::Future: Send + 'static,
    {
        // Convert futures 0.1 to futures 0.3
        let future = async move {
            use futures::compat::Future01CompatExt;
            let _ = r.into_future().compat().await;
        };
        self.spawn_03(future);
    }
}

// Keep the old future::Executor trait implementation for compatibility
impl<F> futures01::future::Executor<F> for Executor
where
    F: futures01::Future<Item = (), Error = ()> + Send + 'static,
{
    fn execute(&self, future: F) -> Result<(), futures01::future::ExecuteError<F>> {
        match self.inner {
            Mode::Tokio(ref executor) => {
                let future_03 = async move {
                    use futures::compat::Future01CompatExt;
                    let _ = future.compat().await;
                };
                executor.spawn(future_03);
                Ok(())
            }
            Mode::Sync => {
                let future_03 = async move {
                    use futures::compat::Future01CompatExt;
                    let _ = future.compat().await;
                };
                create_sync_runtime().block_on(future_03);
                Ok(())
            }
            Mode::ThreadPerFuture => {
                thread::spawn(move || {
                    let future_03 = async move {
                        use futures::compat::Future01CompatExt;
                        let _ = future.compat().await;
                    };
                    get_thread_per_future_runtime().block_on(future_03);
                });
                Ok(())
            }
        }
    }
}

/// A handle to a runtime. Dropping the handle will cause runtime to shutdown.
pub struct RuntimeHandle {
    close: Option<tokio::sync::oneshot::Sender<()>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl From<Runtime> for RuntimeHandle {
    fn from(el: Runtime) -> Self {
        el.handle
    }
}

impl Drop for RuntimeHandle {
    fn drop(&mut self) {
        if let Some(close) = self.close.take() {
            let _ = close.send(());
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl RuntimeHandle {
    /// Blocks current thread and waits until the runtime is finished.
    pub fn wait(mut self) -> thread::Result<()> {
        self.handle
            .take()
            .expect("Handle is taken only in `wait`, `wait` is consuming; qed")
            .join()
    }

    /// Finishes this runtime.
    pub fn close(mut self) {
        let _ = self
            .close
            .take()
            .expect("Close is taken only in `close` and `drop`. `close` is consuming; qed")
            .send(());
    }
}
