use crate::{backend::Backend, dispatcher::Dispatcher};
use tokio::time::timeout;
use awc::Client;
use std::{sync::Arc, time::{Duration, Instant}};

pub async fn measure_latency(backend: &Backend, timeout_duration: Duration) {
  let client = Client::default();
  let start = Instant::now();

  let result = timeout(timeout_duration, client.get(&backend.address).send()).await;

  let latency = match result {
      Ok(Ok(_)) => start.elapsed(),
      _ => Duration::from_secs(u64::MAX),  // Assign a very high latency if the backend is unreachable
  };

  let mut latency_lock = backend.latency.lock().unwrap();
  *latency_lock = latency;
}

pub async fn update_latency(dispatcher: Arc<Dispatcher>, interval: Duration) {
  loop {
    let backends = dispatcher.backends.clone(); // Clone Arc to increment reference count
    for backend in backends.iter() {
        measure_latency(backend, Duration::from_secs(2)).await;
    }
    tokio::time::sleep(interval).await;
  }
}

pub fn least_latency(healthy_backends: Vec<&Backend>) -> &Backend {
  // Select the backend with the lowest latency
  let backend = healthy_backends
      .iter()
      .min_by_key(|backend| *backend.latency.lock().unwrap())
      .unwrap();

  backend
}