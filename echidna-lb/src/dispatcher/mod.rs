use crate::backend::Backend;
use simple_error::SimpleError;
use log;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::sync::atomic::Ordering;
use algorithms::{ip_hashing, round_robin, weighted_round_robin, least_connections, least_latency};

pub mod algorithms;

pub struct Dispatcher {
    pub backends: std::sync::Arc<Vec<Backend>>,
    pub algorithm: LoadBalancingAlgorithm,
    pub current: std::sync::atomic::AtomicUsize,
}

#[derive(Clone, PartialEq)]
pub enum LoadBalancingAlgorithm {
    RoundRobin,
    LeastConnections,
    IPHashing,
    WeightedRoundRobin,
    LeastLatency
}

impl Dispatcher {
    pub async fn dispatch(&self, req: HttpRequest, body: web::Bytes) -> impl Responder {
        // Select backend server based on the algorithm
        let backend = match self.select_backend(&req).await {
            Ok(backend) => backend,
            Err(e) => {
                log::error!("Failed to select a backend server, {}", e.to_string());
                return Ok(())
            }
        };

        log::debug!("Selected backend: {}", backend.address);

        // Increment active connections count
        backend.active_connections.fetch_add(1, Ordering::SeqCst);

        // Forward the request to the selected backend server
        let client = awc::Client::default();
        let backend_url = format!("{}{}", backend.address, req.uri().path_and_query().map_or("", |x| x.as_str()));

        log::debug!("Forwarding request to: {}", backend_url);
        for (header_name, header_value) in req.headers().iter() {
            log::debug!("Request header: {}: {:?}", header_name, header_value);
        }

        let mut forward_req = client.request_from(backend_url.clone(), req.head());

        for (header_name, header_value) in req.headers().iter() {
            forward_req = forward_req.insert_header((header_name.clone(), header_value.clone()));
        }

        let res = forward_req.send_body(body).await;

        // Decrement active connections count
        backend.active_connections.fetch_sub(1, Ordering::SeqCst);

        match res {
            Ok(mut backend_res) => {
                log::debug!("Received response from backend: {}", backend_url);
                log::debug!("Response status: {}", backend_res.status());
                let mut client_res = HttpResponse::build(backend_res.status());
                for (header_name, header_value) in backend_res.headers().iter() {
                    client_res.insert_header((header_name.clone(), header_value.clone()));
                }
                client_res.body(
                    backend_res
                        .body()
                        .await
                        .unwrap_or_else(|_| web::Bytes::new()),
                )
            }
            Err(e) => {
                log::error!("Error forwarding request to backend, {}", e.to_string());
                HttpResponse::InternalServerError().finish()
            },
        }
    }

    async fn select_backend<'l>(&'l self, req: &'l HttpRequest) -> Result<&'l Backend, SimpleError> {
        let healthy_backends: Vec<&Backend> = self.backends.iter()
            .filter(|backend| backend.is_healthy.load(Ordering::SeqCst))
            .collect();

        if healthy_backends.is_empty() {
            return Err(SimpleError::new("No healthy backends available"));
        }

        match self.algorithm {
            LoadBalancingAlgorithm::RoundRobin => {
                Ok(round_robin::round_robin(self, healthy_backends))
            }

            LoadBalancingAlgorithm::LeastConnections => {
                Ok(least_connections::least_connections(healthy_backends))
            }

            LoadBalancingAlgorithm::WeightedRoundRobin => {
                Ok(weighted_round_robin::weighted_round_robin(healthy_backends))
            }

            LoadBalancingAlgorithm::IPHashing => {
                Ok(ip_hashing::ip_hashing(self, req, healthy_backends))
            }

            LoadBalancingAlgorithm::LeastLatency => {
                Ok(least_latency::least_latency(healthy_backends))
            }
        }
    }
}
