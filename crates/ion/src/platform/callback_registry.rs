use std::collections::HashMap;

use parking_lot::Mutex;

#[derive(Default)]
pub struct CallbackRegistry {
    pub worker_shutdown: Mutex<Vec<Box<dyn Send + Sync + FnOnce()>>>,
    pub context_shutdown: Mutex<HashMap<usize, Box<dyn Send + Sync + FnOnce()>>>,
}

impl std::fmt::Debug for CallbackRegistry {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("CallbackRegistry")
            .field("worker_shutdown", &self.worker_shutdown.lock().len())
            .field("context_shutdown", &self.context_shutdown.lock().len())
            .finish()
    }
}
