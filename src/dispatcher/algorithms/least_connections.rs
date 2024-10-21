use crate::backend::Backend;
use std::sync::atomic::Ordering;

pub fn least_connections(healthy_backends: Vec<&Backend>) -> &Backend {
    let mut index = 0;

    for (i, backend) in healthy_backends.iter().enumerate() {
        let active_connections = backend.active_connections.load(Ordering::SeqCst);
        if active_connections
            < healthy_backends[index]
                .active_connections
                .load(Ordering::SeqCst)
        {
            index = i
        }
    }

    healthy_backends[index]
}
