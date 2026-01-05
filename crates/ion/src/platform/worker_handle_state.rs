use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use parking_lot::Mutex;

pub type WorkerShutdownCallback = Box<dyn Send + Sync + FnOnce()>;
pub type ContextShutdownCallback = Box<dyn Send + Sync + FnOnce()>;

pub struct WorkerHandleState {
    pub(crate) worker_handle_active: AtomicBool,
    // pub (crate) context_handle_active: RwLock<HashMap<usize, bool>>,
    pub(crate) worker_shutdown: Mutex<Vec<WorkerShutdownCallback>>,
    pub(crate) context_shutdown: Mutex<HashMap<usize, Vec<ContextShutdownCallback>>>,
}

impl Default for WorkerHandleState {
    fn default() -> Self {
        Self {
            worker_handle_active: AtomicBool::new(true),
            // context_handle_active: Default::default(),
            worker_shutdown: Default::default(),
            context_shutdown: Default::default(),
        }
    }
}

impl WorkerHandleState {
    pub(crate) fn worker_handle_active(&self) -> bool {
        self.worker_handle_active.load(Ordering::Relaxed)
    }

    pub(crate) fn worker_handle_deactivate(&self) {
        self.worker_handle_active.swap(false, Ordering::Relaxed);
    }

    // pub(crate) fn context_handle_active(
    //     &self,
    //     id: &usize,
    // ) -> bool {
    //     self.context_handle_active
    //         .read()
    //         .get(id)
    //         .unwrap_or(&false)
    //         .clone()
    // }

    // pub(crate) fn context_handle_set_status(
    //     &self,
    //     id: &usize,
    //     status: bool,
    // ) {
    //     self.context_handle_active
    //         .write()
    //         .insert(id.clone(), status);
    // }

    // pub(crate) fn add_worker_shutdown_callback(
    //     &self,
    //     callback: impl 'static + Send + Sync + FnOnce(),
    // ) {
    //     self.worker_shutdown.lock().push(Box::new(callback));
    // }

    // pub(crate) fn add_context_shutdown_callback(
    //     &self,
    //     id: usize,
    //     callback: impl 'static + Send + Sync + FnOnce(),
    // ) {
    //     self.context_shutdown
    //         .lock()
    //         .entry(id)
    //         .or_default()
    //         .push(Box::new(callback));
    // }

    // pub(crate) fn take_worker_shutdown_callbacks(&self) -> Vec<WorkerShutdownCallback> {
    //     std::mem::take(&mut *self.worker_shutdown.lock())
    // }

    // pub(crate) fn take_context_shutdown_callbacks(
    //     &self,
    //     id: usize,
    // ) -> Vec<ContextShutdownCallback> {
    //     std::mem::take(&mut *self.context_shutdown.lock().entry(id).or_default())
    // }
}

impl std::fmt::Debug for WorkerHandleState {
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
