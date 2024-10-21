use crate::dispatcher::algorithms::round_robin::round_robin;
use crate::{backend::Backend, dispatcher::Dispatcher};
use actix_web::HttpRequest;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

pub fn ip_hashing<'l>(
    dispatcher: &'l Dispatcher,
    req: &'l HttpRequest,
    healthy_backends: Vec<&'l Backend>,
) -> &'l Backend {
    if let Some(peer_addr) = req.peer_addr() {
        let ip_str = peer_addr.ip().to_string();
        let mut hasher = DefaultHasher::new();
        ip_str.hash(&mut hasher);
        let hash = hasher.finish();
        let idx = (hash % healthy_backends.len() as u64) as usize;
        healthy_backends[idx]
    } else {
        // Fallback to round-robin if the client IP cannot be determined
        round_robin(dispatcher, healthy_backends)
    }
}
