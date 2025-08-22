use std::sync::Arc;

use flume::Sender;
use flume::bounded;

use super::Error;
use super::JsWorker;
use super::platform::PlatformEvent;

/// JsRuntime is a handle to the underlying v8 engine. It can spawn worker threads and
/// evaluate JavaScript within Contexts
#[derive(Clone, Debug)]
pub struct JsRuntime {
    pub(crate) tx: Sender<super::platform::PlatformEvent>,
}

impl JsRuntime {
    /// Spawns a dedicated worker thread for isolates
    pub fn spawn_worker(&self) -> super::Result<Arc<JsWorker>> {
        let (tx, rx) = bounded(1);

        if self.tx.send(PlatformEvent::SpawnWorker(tx)).is_err() {
            return Err(Error::WorkerInitializeError);
        };

        let Ok(worker) = rx.recv() else {
            return Err(Error::WorkerInitializeError);
        };

        Ok(worker)
    }
}

impl Drop for JsRuntime {
    fn drop(&mut self) {
        // Once initialized, JsRuntime is never dropped
        // Only the worker threads and isolates are dropped
    }
}
