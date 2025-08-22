use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;
use std::usize;

use flume::Sender;
use flume::bounded;
use flume::unbounded;

use crate::Error;
use crate::utils::channel::oneshot;

use super::Env;
use super::JsContext;

/// This is a handle to a v8::Isolate running on a dedicated thread.
/// A worker thread can spawn multiple v8::Contexts within that thread
/// to be used to execute JavaScript
#[derive(Debug)]
pub struct JsWorker {
    tx: Sender<JsWorkerEvent>,
    handle: Mutex<Option<JoinHandle<()>>>,
}

pub(crate) enum JsWorkerEvent {
    CreateContext(Sender<Arc<JsContext>>),
    ShutdownContext(usize, Sender<()>),
    Exec(usize, Box<dyn Send + FnOnce(Env) -> crate::Result<()>>),
    Shutdown(Sender<()>),
}

impl JsWorker {
    pub(crate) fn new() -> Self {
        let (tx, rx) = unbounded::<JsWorkerEvent>();

        // Create a dedicated thread to host the isolate
        let handle = thread::spawn({
            let tx = tx.clone();

            move || {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(async {
                        // Maintain a store of contexts to help with cleanup on shutdown.
                        let mut contexts = HashMap::<usize, Env>::new();

                        // Create an isolate dedicated to this "worker" thread
                        let mut isolate = v8::Isolate::new(v8::CreateParams::default());
                        let isolate_ptr = isolate.as_mut() as *mut v8::Isolate;

                        while let Ok(event) = rx.recv_async().await {
                            match event {
                                JsWorkerEvent::CreateContext(tx_resolve) => {
                                    let env = Env::new(isolate_ptr);
                                    let id = env.id();

                                    contexts.insert(id.clone(), env);

                                    tx_resolve
                                        .try_send(Arc::new(JsContext {
                                            id: id,
                                            tx: tx.clone(),
                                        }))
                                        .unwrap();
                                }
                                // This event removes a context and cleans up memory
                                JsWorkerEvent::ShutdownContext(id, tx_resolve) => {
                                    let env = contexts.remove(&id).unwrap();

                                    loop {
                                        let tasks = env.tasks.take();
                                        let Some(tasks) = tasks else { break };
                                        tasks.await;
                                    }

                                    // SAFETY: These values are allocated under CreateContext
                                    unsafe {
                                        drop(Box::from_raw(
                                            env.context_scope
                                                as *mut v8::ContextScope<'_, v8::HandleScope<'_>>,
                                        ));
                                        drop(Box::from_raw(
                                            env.context as *mut v8::Global<v8::Context>,
                                        ));
                                        drop(Box::from_raw(
                                            env.handle_scope as *mut v8::HandleScope<'_, ()>,
                                        ));
                                    }

                                    tx_resolve.try_send(()).unwrap();
                                }
                                JsWorkerEvent::Exec(id, callback) => {
                                    let env = contexts.get_mut(&id).unwrap();
                                    if let Err(err) = callback(env.clone()) {
                                        // TODO
                                        panic!("Callback errored {:?}", err)
                                    };
                                }
                                JsWorkerEvent::Shutdown(tx_resolve) => {
                                    // TODO consolidate context cleanup, might not be sound if there are
                                    // references to workers/contexts in multiple threads
                                    for (_id, env) in contexts {
                                        // Wait on any async tasks to complete before terminating worker
                                        loop {
                                            let tasks = env.tasks.take();
                                            let Some(tasks) = tasks else { break };
                                            tasks.await;
                                        }

                                        // SAFETY: These values are allocated under CreateContext
                                        unsafe {
                                            drop(Box::from_raw(
                                                env.context_scope
                                                    as *mut v8::ContextScope<
                                                        '_,
                                                        v8::HandleScope<'_>,
                                                    >,
                                            ));
                                            drop(Box::from_raw(
                                                env.context as *mut v8::Global<v8::Context>,
                                            ));
                                            drop(Box::from_raw(
                                                env.handle_scope as *mut v8::HandleScope<'_, ()>,
                                            ));
                                        }
                                    }
                                    tx_resolve.try_send(()).unwrap();
                                    break;
                                }
                            }
                        }
                    });
            }
        });

        JsWorker {
            tx,
            handle: Mutex::new(Some(handle)),
        }
    }

    /// Create a handle to a v8::Context associated with this v8::Isolate
    pub fn create_context(&self) -> crate::Result<Arc<JsContext>> {
        let (tx, rx) = bounded(1);

        if self.tx.send(JsWorkerEvent::CreateContext(tx)).is_err() {
            return Err(Error::WorkerInitializeError);
        };

        let Ok(context) = rx.recv() else {
            return Err(Error::WorkerInitializeError);
        };

        Ok(context)
    }
}

impl Drop for JsWorker {
    fn drop(&mut self) {
        let (tx, rx) = oneshot();
        self.tx.send(JsWorkerEvent::Shutdown(tx)).unwrap();
        rx.recv().unwrap();
        let mut handle = self.handle.lock().unwrap();
        drop(handle.take().unwrap());
    }
}
