use crate::backend::Backend;

pub fn weighted_round_robin(healthy_backends: Vec<&Backend>) -> &Backend {
    let mut max_weight = -1;
    let mut total_weight: isize = 0;
    let mut selected_backend = healthy_backends[0];

    for backend in healthy_backends.iter() {
        let mut current_weight: std::sync::MutexGuard<isize> = backend.current_weight.lock().unwrap();
        *current_weight += backend.weight as isize;

        if *current_weight > max_weight {
            max_weight = *current_weight;
            selected_backend = backend;
        }

        total_weight += backend.weight as isize;
    }

    let mut selected_weight = selected_backend.current_weight.lock().unwrap();
    *selected_weight -= total_weight;

    selected_backend
}
