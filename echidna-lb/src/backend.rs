use std::time::Duration;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;

use crate::dispatcher::Dispatcher;

#[derive(Clone)]
pub struct Backend {
    pub address: String,
    pub weight: usize,
    pub active_connections: Arc<AtomicUsize>,
    pub is_healthy: Arc<AtomicBool>,
    pub current_weight: Arc<Mutex<isize>>,
    pub latency: Arc<Mutex<Duration>>,
}

pub async fn health_check(dispatcher: Arc<Dispatcher>, interval: Duration, mut route: String) {
    let client = awc::Client::default();
    if !route.starts_with('/') {
        route = "/".to_string() + &route.clone();
    }

    loop {
        for backend in dispatcher.backends.iter() {
            let health_check_url = format!("{}{}", backend.address, route);
            let response = client.get(health_check_url).send().await;

            match response {
                Ok(res) => {
                    if res.status().is_success() {
                        backend.is_healthy.store(true, Ordering::SeqCst);
                    } else {
                        backend.is_healthy.store(false, Ordering::SeqCst);
                    }
                }
                Err(_) => {
                    backend.is_healthy.store(false, Ordering::SeqCst);
                }
            }
        }

        tokio::time::sleep(interval).await;
    }
}