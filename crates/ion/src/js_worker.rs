use std::sync::Arc;
use std::sync::Mutex;
use std::thread::JoinHandle;

use flume::Sender;
use flume::bounded;

use super::JsContext;
use crate::Error;
use crate::JsExtension;
use crate::JsResolver;
use crate::JsTransformer;
use crate::platform::callback_registry::CallbackRegistry;
use crate::platform::worker::JsWorkerEvent;
use crate::utils::channel::oneshot;

#[derive(Default)]
pub struct JsWorkerOptions {
    /// Hook that runs before code is imported. This can be used to
    /// customize the behavior of "import" statements
    pub resolvers: Vec<JsResolver>,
    /// Hook that runs before code is loaded. This can be used to
    /// convert TypeScript into JavaScript or JSON into JavaScript
    pub transformers: Vec<JsTransformer>,
    /// Extensions that will be available to all [`crate::JsWorker`] and [`crate::JsContext`] instances
    pub extensions: Vec<JsExtension>,
}

/// This is a handle to a v8::Isolate running on a dedicated thread.
/// A worker thread can spawn multiple v8::Contexts within that thread
/// to be used to execute JavaScript
#[derive(Debug)]
pub struct JsWorker {
    callback_registry: Arc<CallbackRegistry>,
    tx: Sender<JsWorkerEvent>,
    handle: Arc<Mutex<Option<JoinHandle<crate::Result<()>>>>>,
}

impl JsWorker {
    pub(crate) fn new(
        callback_registry: Arc<CallbackRegistry>,
        tx: Sender<JsWorkerEvent>,
        handle: Arc<Mutex<Option<JoinHandle<crate::Result<()>>>>>,
    ) -> Self {
        JsWorker {
            tx,
            handle,
            callback_registry,
        }
    }

    /// Create a handle to a v8::Context associated with this v8::Isolate
    pub fn create_context(&self) -> crate::Result<JsContext> {
        let (tx, rx) = bounded(1);

        if self
            .tx
            .send(JsWorkerEvent::CreateContext { resolve: tx })
            .is_err()
        {
            return Err(Error::WorkerInitializeError);
        };

        let Ok((id, tx)) = rx.recv() else {
            return Err(Error::WorkerInitializeError);
        };

        self.callback_registry.context_handle_set_status(&id, true);

        Ok(JsContext {
            id,
            tx,
            callback_registry: Arc::clone(&self.callback_registry),
        })
    }

    pub fn run_garbage_collection_for_testing(&self) -> crate::Result<()> {
        let (tx, rx) = bounded(1);

        if self
            .tx
            .send(JsWorkerEvent::RunGarbageCollectionForTesting { resolve: tx })
            .is_err()
        {
            return Err(Error::WorkerInitializeError);
        };

        Ok(rx.recv()?)
    }

    /// Wait for all of the contexts within the worker to complete all activity
    pub fn join_blocking(self) -> crate::Result<()> {
        self.callback_registry.worker_handle_deactivate();

        let (tx, rx) = oneshot();
        self.callback_registry
            .add_worker_shutdown_callback(move || {
                tx.send(()).unwrap();
            });

        self.tx
            .send(JsWorkerEvent::WorkerHandleDeactivated)
            .unwrap();

        if rx.recv().is_err() {
            panic!("Cannot drop JsWorker 2");
        }

        let Ok(mut handle) = self.handle.lock() else {
            panic!("Cannot drop JsWorker 3");
        };

        if let Some(handle) = handle.take() {
            drop(handle.join().unwrap());
        }

        Ok(())
    }

    /// Wait for all of the contexts within the worker to complete all activity
    pub async fn join_async(self) -> crate::Result<()> {
        Ok(())
    }
}

impl Drop for JsWorker {
    fn drop(&mut self) {
        self.callback_registry.worker_handle_deactivate();
        drop(self.tx.try_send(JsWorkerEvent::WorkerHandleDropped));
    }
}
