use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;
use std::usize;

use flume::Sender;
use flume::bounded;
use flume::unbounded;

use crate::Error;
use crate::event_loop::EventLoop;
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
    SpinEventLoop,
}

impl JsWorker {
    pub(crate) fn new() -> Self {
        let (tx, rx) = unbounded::<JsWorkerEvent>();

        // Create a dedicated thread to host the isolate
        let handle = thread::spawn({
            let tx = tx.clone();

            move || {
                // The event loop is controlled using a custom async runtime.
                // This requires a dedicated thread to listen for wake events.
                // The waker is initialized once per worker/isolate however the
                // event loop is partitioned by context.
                let waker = EventLoop::start_waker_thread({
                    let tx = tx.clone();
                    // Callback runs when a task in the event loop is ready to progress
                    move || tx.send(JsWorkerEvent::SpinEventLoop).unwrap()
                });

                // Maintain a store of contexts to help with cleanup on shutdown.
                let mut contexts = HashMap::<usize, Env>::new();

                // Create an isolate dedicated to this "worker" thread
                let mut isolate = v8::Isolate::new(v8::CreateParams::default());
                let isolate_ptr = isolate.as_mut() as *mut v8::Isolate;

                // This thread is interacted with via the JsWorker handle. The handle itself
                // can be cloned and is Send + Sync, however all calls to the isolate occur
                // via a channel.
                while let Ok(event) = rx.recv() {
                    match event {
                        // One isolate has many contexts. This event creates/tracks each context
                        JsWorkerEvent::CreateContext(tx_resolve) => {
                            let event_loop = Rc::new(RefCell::new(EventLoop::new(waker.clone())));

                            let handle_scope = v8::HandleScope::new(&mut isolate);

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
                                event_loop,
                            };

                            contexts.insert(global_context_ptr as usize, env);

                            tx_resolve
                                .send(Arc::new(JsContext {
                                    id: global_context_ptr as usize,
                                    tx: tx.clone(),
                                }))
                                .unwrap();
                        }
                        // This event removes a context and cleans up memory
                        JsWorkerEvent::ShutdownContext(id, resolve) => {
                            let env = contexts.remove(&id).unwrap();

                            // Wait on any async tasks to complete before exiting context
                            let mut event_loop = env.event_loop.borrow_mut();
                            if event_loop.run_to_completion().is_err() {
                                panic!("Error completing event loop")
                            };

                            // SAFETY: These values are allocated under CreateContext
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

                            resolve.send(()).expect("Error resolving worker event");
                        }
                        // This even runs when the JsRuntime is dropped and cleans up all contexts
                        JsWorkerEvent::Shutdown(resolve) => {
                            for (_id, env) in contexts {
                                // Wait on any async tasks to complete before terminating worker
                                let mut event_loop = env.event_loop.borrow_mut();
                                if event_loop.run_to_completion().is_err() {
                                    panic!("Error completing event loop")
                                };

                                // TODO consolidate context cleanup
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
                            }
                            resolve.send(()).expect("Error resolving worker event");
                            break;
                        }
                        // This event is how consumers interact with the internal runtime
                        JsWorkerEvent::Exec(id, callback) => {
                            let env = contexts.get_mut(&id).unwrap();
                            if let Err(err) = callback(env.clone()) {
                                // TODO
                                panic!("Callback errored {:?}", err)
                            };
                        }
                        // This event progresses the event loop and exits when no progress can be made
                        JsWorkerEvent::SpinEventLoop => {
                            for (_id, env) in &contexts {
                                let mut event_loop = env.event_loop.borrow_mut();
                                if event_loop.run_until_stalled().is_err() {
                                    panic!("Error executing event loop")
                                };
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
        let mut handle = self.handle.lock().unwrap();
        drop(handle.take().unwrap());
        self.tx.send(JsWorkerEvent::Shutdown(tx)).unwrap();
        rx.recv().unwrap();
    }
}
