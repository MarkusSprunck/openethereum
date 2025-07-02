use std::{convert::Infallible, sync::Arc, time::Instant};
use crate::{rpc, rpc_apis};
use parking_lot::Mutex;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server, StatusCode,
};
use stats::{
    prometheus::{self, Encoder},
    PrometheusMetrics, PrometheusRegistry,
};

#[derive(Debug, Clone, PartialEq)]
pub struct MetricsConfiguration {
    /// Are metrics enabled (default is false)?
    pub enabled: bool,
    /// Prefix
    pub prefix: String,
    /// The IP of the network interface used (default is 127.0.0.1).
    pub interface: String,
    /// The network port (default is 3000).
    pub port: u16,
}

impl Default for MetricsConfiguration {
    fn default() -> Self {
        MetricsConfiguration {
            enabled: false,
            prefix: "".into(),
            interface: "127.0.0.1".into(),
            port: 3000,
        }
    }
}

struct State {
    rpc_apis: Arc<rpc_apis::FullDependencies>,
}

async fn handle_request(
    req: Request<Body>,
    conf: Arc<MetricsConfiguration>,
    state: Arc<Mutex<State>>,
) -> Result<Response<Body>, Infallible> {
    let (parts, _body) = req.into_parts();
    
    match (parts.method, parts.uri.path()) {
        (Method::GET, "/metrics") => {
            let start = Instant::now();
            let mut reg = PrometheusRegistry::new(conf.prefix.clone());
            
            let state = state.lock();
            
            // If these are synchronous calls (most likely case)
            state.rpc_apis.client.prometheus_metrics(&mut reg);
            state.rpc_apis.sync.prometheus_metrics(&mut reg);
            
            // If they return futures 0.1, uncomment and use this instead:
            // if let Ok(client_future) = std::panic::catch_unwind(|| {
            //     state.rpc_apis.client.prometheus_metrics_async(&mut reg)
            // }) {
            //     if let Ok(_) = client_future.compat().await {
            //         // metrics collected successfully
            //     }
            // }
            
            let elapsed = start.elapsed();
            reg.register_gauge(
                "metrics_time",
                "Time to perform rpc metrics",
                elapsed.as_millis() as i64,
            );

            let mut buffer = vec![];
            let encoder = prometheus::TextEncoder::new();
            let metric_families = reg.registry().gather();
            encoder
                .encode(&metric_families, &mut buffer)
                .expect("all source of metrics are static; qed");

            let text = String::from_utf8(buffer).expect("metrics encoding is ASCII; qed");
            Ok(Response::new(Body::from(text)))
        }
        (_, _) => {
            let mut res = Response::new(Body::from("not found"));
            *res.status_mut() = StatusCode::NOT_FOUND;
            Ok(res)
        }
    }
}

/// Start the prometheus metrics server accessible via GET :/metrics
pub fn start_prometheus_metrics(
    conf: &MetricsConfiguration,
    deps: &rpc::Dependencies<rpc_apis::FullDependencies>,
) -> Result<(), String> {
    if !conf.enabled {
        return Ok(());
    }

    let conf = conf.clone();
    let apis = deps.apis.clone();

    // Spawn in a separate thread with its own tokio runtime
    std::thread::spawn(move || {
        // Create a new tokio runtime for this thread
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("Failed to create tokio runtime for metrics server: {}", e);
                return;
            }
        };

        rt.block_on(async move {
            let addr = format!("{}:{}", conf.interface, conf.port);
            let addr = match addr.parse() {
                Ok(addr) => addr,
                Err(e) => {
                    eprintln!("Failed to parse address '{}': {}", addr, e);
                    return;
                }
            };

            let state = State { rpc_apis: apis };
            let state = Arc::new(Mutex::new(state));
            let conf = Arc::new(conf);

            let make_svc = make_service_fn(move |_conn| {
                let state = state.clone();
                let conf = conf.clone();
                
                async move {
                    Ok::<_, Infallible>(service_fn(move |req| {
                        handle_request(req, conf.clone(), state.clone())
                    }))
                }
            });

            let server = Server::bind(&addr).serve(make_svc);
            
            info!("Started prometheus metrics at http://{}/metrics", addr);

            if let Err(e) = server.await {
                eprintln!("Metrics server error: {}", e);
            }
        });
    });

    Ok(())
}
