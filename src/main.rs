use actix_web::middleware::Logger;
use actix_web::{
    web::{route, Bytes, Data},
    App, HttpRequest, HttpServer, Responder,
};
use backend::{health_check, Backend};
use clap::Parser;
use dispatcher::{
    algorithms::least_latency::update_latency,
    Dispatcher,
    LoadBalancingAlgorithm::{
        IPHashing, LeastConnections, LeastLatency, RoundRobin, WeightedRoundRobin,
    },
};
use error::EchidnaError::Custom;
use std::io;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tls::load_tls_config;

pub mod backend;
pub mod config;
pub mod dispatcher;
mod error;
pub mod tls;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let args = config::Args::parse();

    let config = match config::load_config(args.config.as_path()) {
        Ok(config) => config,
        Err(e) => {
            log::error!(
                "Failed to parse config at {}, {}",
                args.config.to_string_lossy(),
                e
            );
            return Err(io::Error::new(io::ErrorKind::Other, e));
        }
    };

    std::env::set_var("RUST_LOG", "info");
    if let Some(debug) = config.debug {
        if debug {
            std::env::set_var("RUST_LOG", "debug");
        }
    }

    env_logger::init();

    let backends: Vec<Backend> = config
        .backends
        .into_iter()
        .map(|b| Backend {
            address: b.url,
            weight: b.weight.unwrap_or(1),
            active_connections: Arc::new(AtomicUsize::new(0)),
            is_healthy: Arc::new(AtomicBool::new(true)),
            current_weight: Arc::new(Mutex::new(0)),
            latency: Arc::new(Mutex::new(Duration::from_secs(u64::MAX))),
        })
        .collect();

    let algorithm = match config.algorithm.as_deref() {
        Some("RoundRobin") | None => RoundRobin,
        Some("LeastConnections") => LeastConnections,
        Some("WeightedRoundRobin") => WeightedRoundRobin,
        Some("IPHashing") => IPHashing,
        Some("LeastLatency") => LeastLatency,
        _ => return Err(Custom(format!("Unknown algorithm: {:?}", config.algorithm)).into()),
    };

    let dispatcher = Arc::new(Dispatcher {
        backends: Arc::new(backends),
        algorithm: algorithm.clone(),
        current: AtomicUsize::new(0),
    });

    if let Some(healthcheck_config) = config.healthcheck {
        let dispatcher_clone = dispatcher.clone();
        actix_rt::spawn(async move {
            health_check(
                dispatcher_clone,
                Duration::from_secs(healthcheck_config.interval_sec.unwrap_or(10)),
                healthcheck_config.route,
            )
            .await;
        });
    }

    if algorithm == LeastLatency {
        // Spawn the latency update task
        let dispatcher_clone = dispatcher.clone();
        actix_rt::spawn(async move {
            update_latency(dispatcher_clone, Duration::from_secs(10)).await;
        });
    }

    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(dispatcher.clone()))
            .wrap(Logger::default())
            .default_service(route().to(dispatch_request))
    })
    .workers(config.workers.unwrap_or(10));

    if let Some(ssl_config) = &config.ssl {
        let tls_config = match load_tls_config(ssl_config) {
            Ok(tls_config) => tls_config,
            Err(e) => {
                log::error!("Failed to parse TLS config, {}", e);
                return Err(io::Error::new(io::ErrorKind::Other, e));
            }
        };

        server
            .bind(("0.0.0.0", config.port.unwrap_or(9000)))?
            .bind(("::", config.port.unwrap_or(9000)))?
            .bind_rustls(
                ("0.0.0.0", config.https_port.unwrap_or(9001)),
                tls_config.clone(),
            )?
            .bind_rustls(
                ("::", config.https_port.unwrap_or(9001)),
                tls_config.clone(),
            )?
            .run()
            .await
    } else {
        server
            .bind(("0.0.0.0", config.port.unwrap_or(9000)))?
            .bind(("::", config.port.unwrap_or(9000)))?
            .run()
            .await
    }
}

async fn dispatch_request(
    dispatcher: Data<Arc<Dispatcher>>,
    req: HttpRequest,
    body: Bytes,
) -> impl Responder {
    dispatcher.dispatch(req, body).await
}
