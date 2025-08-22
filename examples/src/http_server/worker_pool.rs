use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use ion::JsContext;
use ion::JsRuntime;
use ion::JsWorker;

// Basic load balancer, round robin
pub struct WorkerPool {
    current_index: AtomicUsize,
    workers: Vec<Arc<JsWorker>>,
}

impl WorkerPool {
    pub fn new(
        runtime: &JsRuntime,
        worker_count: usize,
    ) -> Self {
        let mut workers = vec![];

        for _ in 0..worker_count {
            workers.push(runtime.spawn_worker().unwrap());
        }

        Self {
            current_index: Default::default(),
            workers,
        }
    }

    fn next_index(&self) -> usize {
        self.current_index
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                Some((value + 1) % self.workers.len())
            })
            .expect("Could not get worker")
    }

    pub fn get_worker(&self) -> &JsWorker {
        let next = self.next_index();
        // println!("Worker {}", next);
        &self.workers[next]
    }

    pub fn create_context(&self) -> Arc<JsContext> {
        let worker = self.get_worker();
        worker.create_context().unwrap()
    }
}
