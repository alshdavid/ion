use std::ffi::c_void;
use std::sync::Arc;
use std::{collections::HashMap, thread::JoinHandle};
use std::{thread, usize};

use flume::{Sender, bounded, unbounded};
use parking_lot::Mutex;

use crate::Error;

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
    SpawnContext(Sender<Arc<JsContext>>),
    ShutdownContext(usize),
    Exec(usize, Box<dyn Send + FnOnce(Env) -> crate::Result<()>>),
    Shutdown,
}

impl JsWorker {
    pub(crate) fn new() -> Self {
        let (tx, rx) = unbounded::<JsWorkerEvent>();

        let handle = thread::spawn({
            let tx = tx.clone();

            move || {
                let mut contexts = HashMap::<usize, Env>::new();

                let mut isolate = v8::Isolate::new(v8::CreateParams::default());
                let isolate_ptr = isolate.as_mut() as *mut v8::Isolate;

                while let Ok(event) = rx.recv() {
                    match event {
                        JsWorkerEvent::SpawnContext(tx_resolve) => {
                            let mut handle_scope = v8::HandleScope::new(&mut isolate);

                            let handle_scope_ptr = Box::new(handle_scope);
                            let handle_scope_ptr = Box::into_raw(handle_scope_ptr);

                            let context = v8::Context::new(
                                unsafe { &mut *handle_scope_ptr },
                                Default::default(),
                            );
                            let mut context_scope =
                                v8::ContextScope::new(unsafe { &mut *handle_scope_ptr }, context);

                            let global_context = v8::Global::new(&mut context_scope, context);
                            let global_context_ptr = Box::into_raw(Box::new(global_context));

                            let context_scope_ptr = Box::new(context_scope);
                            let context_scope_ptr = Box::into_raw(context_scope_ptr);

                            let env = Env {
                                isolate: isolate_ptr,
                                handle_scope: handle_scope_ptr as _,
                                context: global_context_ptr as _,
                                context_scope: context_scope_ptr as _,
                                scope: std::ptr::null::<c_void>() as _,
                            };

                            contexts.insert(global_context_ptr as usize, env);

                            tx_resolve.send(Arc::new(JsContext {
                                id: global_context_ptr as usize,
                                tx: tx.clone(),
                            }));
                        }
                        JsWorkerEvent::ShutdownContext(id) => {
                            let env = contexts.remove(&id).unwrap();

                            unsafe {
                                drop(Box::from_raw(
                                    env.context_scope
                                        as *mut v8::ContextScope<'_, v8::HandleScope<'_>>,
                                ));
                                drop(Box::from_raw(env.context as *mut v8::Global<v8::Context>));
                                drop(Box::from_raw(
                                    env.handle_scope as *mut v8::HandleScope<'_, ()>,
                                ));
                            }
                        }
                        JsWorkerEvent::Shutdown => {
                            for (id, env) in contexts {
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
                            }
                            break;
                        }
                        JsWorkerEvent::Exec(id, callback) => {
                            let env = contexts.get_mut(&id).unwrap();
                            let scope = Box::new(v8::HandleScope::new(env.context_scope()));
                            let scope_ptr = Box::into_raw(scope);
                            env.scope = scope_ptr as _;

                            callback(env.clone());

                            unsafe {
                                drop(Box::from_raw(env.scope as *mut v8::HandleScope<'_>));
                            }
                        }
                    }
                }
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

        if self.tx.send(JsWorkerEvent::SpawnContext(tx)).is_err() {
            return Err(Error::WorkerInitializeError);
        };

        let Ok(context) = rx.recv() else {
            return Err(Error::WorkerInitializeError);
        };

        Ok(context)
    }

    pub fn shutdown(&self) {
        let mut handle = self.handle.lock();
        let mut handle = handle.take().unwrap();
        self.tx.send(JsWorkerEvent::Shutdown).unwrap();
        handle.join();
    }
}

impl Drop for JsWorker {
    fn drop(&mut self) {
        self.shutdown();
    }
}
