use crate::{backend::Backend, dispatcher::Dispatcher};
use std::sync::atomic::Ordering;

pub fn round_robin<'l>(
    dispatcher: &'l Dispatcher,
    healthy_backends: Vec<&'l Backend>,
) -> &'l Backend {
    let idx = dispatcher.current.fetch_add(1, Ordering::SeqCst) % healthy_backends.len();
    healthy_backends[idx]
}
