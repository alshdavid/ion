use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;

use flume::Receiver;
use flume::Sender;
use flume::unbounded;
use tracing::Span;

use super::JsRealm;
use super::extension::Extension;
use super::module::Module;
use crate::Env;
use crate::JsExtension;
use crate::JsResolver;
use crate::JsTransformer;
use crate::fs::FileSystem;
use crate::platform::background_worker::BackgroundTaskManager;
use crate::platform::worker_handle_state::WorkerHandleState;
use crate::utils::HashMapExt;
use crate::utils::PathExt;
use crate::utils::complete_signal::CompleteSignal;

pub(crate) enum JsWorkerEvent {
    CreateContext {
        context_shutdown_sig: CompleteSignal,
        resolve: Sender<(usize, Sender<JsWorkerEvent>)>,
    },
    Exec {
        id: usize,
        #[allow(clippy::type_complexity)]
        callback: Box<dyn Send + FnOnce(&Env) -> crate::Result<()>>,
        span: Span,
    },
    Import {
        id: usize,
        specifier: String,
    },
    RunGarbageCollectionForTesting {
        resolve: Sender<()>,
    },
    WorkerHandleDropped,
    WorkerHandleDeactivated,
    ContextHandleDropped {
        id: usize,
    },
    ContextHandleDeactivated {
        id: usize,
    },
    BackgroundTaskComplete {
        id: usize,
    },
}

// Create a dedicated thread to host the isolate
#[allow(clippy::type_complexity)]
pub(crate) fn start_js_worker_thread(
    worker_shutdown_sig: CompleteSignal,
    worker_handle_state: Arc<WorkerHandleState>,
    background_task_manager: Arc<BackgroundTaskManager>,
    extensions: Vec<Arc<JsExtension>>,
    resolvers: Vec<JsResolver>,
    transformers: HashMap<String, Arc<JsTransformer>>,
) -> (
    Sender<JsWorkerEvent>,
    Mutex<Option<JoinHandle<crate::Result<()>>>>,
) {
    let (tx, rx) = unbounded::<JsWorkerEvent>();

    // Start a thread for the Isolate
    let handle: JoinHandle<crate::Result<()>> = thread::spawn({
        let tx: Sender<JsWorkerEvent> = tx.clone();
        move || {
            worker_thread(
                worker_shutdown_sig.clone(),
                worker_handle_state,
                tx,
                rx,
                background_task_manager,
                extensions,
                resolvers,
                transformers,
            )
        }
    });

    (tx, Mutex::new(Some(handle)))
}

fn worker_thread(
    worker_shutdown_sig: CompleteSignal,
    worker_handle_state: Arc<WorkerHandleState>,
    tx: Sender<JsWorkerEvent>,
    rx: Receiver<JsWorkerEvent>,
    background_task_manager: Arc<BackgroundTaskManager>,
    extensions: Vec<Arc<JsExtension>>,
    resolvers: Vec<JsResolver>,
    transformers: HashMap<String, Arc<JsTransformer>>,
) -> crate::Result<()> {
    let fs = FileSystem::Physical;

    // One isolate per worker thread
    let mut isolate = v8::Isolate::new(v8::CreateParams::default());
    let isolate_ptr = isolate.as_mut() as *mut v8::Isolate;

    // Maintain a store of Global<Context> to help with cleanup on shutdown.
    let mut realms = HashMap::<usize, Box<JsRealm>>::new();

    while let Ok(event) = rx.recv() {
        println!("  {:?}", event);

        match event {
            JsWorkerEvent::CreateContext {
                resolve,
                context_shutdown_sig,
            } => {
                let realm = JsRealm::new(
                    isolate_ptr,
                    fs.clone(),
                    resolvers.clone(),
                    transformers.clone(),
                    background_task_manager.clone(),
                    tx.clone(),
                    context_shutdown_sig,
                );
                let realm_id = realm.id();

                Extension::register_extensions(&realm, &extensions, &transformers)?;

                realms.insert(realm_id, realm);
                resolve.try_send((realm_id, tx.clone()))?;
            }
            JsWorkerEvent::Exec { id, callback, span } => {
                let realm = realms.try_get(&id)?;

                let _span_guard = span.enter();
                if let Err(err) = callback(realm.env()) {
                    // TODO global error handler
                    panic!("Callback errored {:?}", err)
                };

                if worker_handle_state.context_handle_active(&id)
                {
                    continue;
                }

                if realm.global_refs.count() != 0 {
                    continue;
                }

                let Some(realm) = realms.remove(&id) else {
                    continue;
                };

                let finalizer_registry = realm.finalizer_registry;
                finalizer_registry.clear();
                drop(finalizer_registry);

                realm.context_shutdown_sig.done();
            }
            JsWorkerEvent::BackgroundTaskComplete { id } => {
                let realm = realms.try_get(&id)?;

                if worker_handle_state.context_handle_active(&id)
                {
                    continue;
                }

                if realm.global_refs.count() != 0 {
                    continue;
                }

                let Some(realm) = realms.remove(&id) else {
                    continue;
                };

                let finalizer_registry = realm.finalizer_registry;
                finalizer_registry.clear();
                drop(finalizer_registry);

                realm.context_shutdown_sig.done();
            }
            JsWorkerEvent::Import { id, specifier } => {
                Module::v8_initialize(
                    true,
                    realms.try_get(&id)?,
                    &specifier,
                    std::env::current_dir()?.try_to_string()?,
                )?;
            }
            JsWorkerEvent::RunGarbageCollectionForTesting { resolve } => {
                isolate.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
                resolve.try_send(())?;
            }
            JsWorkerEvent::WorkerHandleDropped => {
                for id in realms.keys().cloned().collect::<Vec<usize>>() {
                    let Some(realm) = realms.remove(&id) else {
                        continue;
                    };

                    let finalizer_registry = realm.finalizer_registry;
                    finalizer_registry.clear();
                    drop(finalizer_registry);

                    realm.context_shutdown_sig.done();
                }

                break;
            }
            JsWorkerEvent::WorkerHandleDeactivated => {
                let mut to_drop = vec![];

                for (id, realm) in realms.iter() {
                    if realm.global_refs.count() != 0 {
                        continue;
                    }
                    to_drop.push(id.clone());
                }

                for id in to_drop {
                    let Some(realm) = realms.remove(&id) else {
                        continue;
                    };

                    let finalizer_registry = realm.finalizer_registry;
                    finalizer_registry.clear();
                    drop(finalizer_registry);

                    realm.context_shutdown_sig.done();
                }
            }
            JsWorkerEvent::ContextHandleDeactivated { id } => {
                if realms.try_get_mut(&id)?.global_refs.count() != 0 {
                    continue;
                }

                let Some(realm) = realms.remove(&id) else {
                    continue;
                };

                let finalizer_registry = realm.finalizer_registry;
                finalizer_registry.clear();
                drop(finalizer_registry);

                realm.context_shutdown_sig.done();
            }
            JsWorkerEvent::ContextHandleDropped { id } => {
                let Some(realm) = realms.remove(&id) else {
                    continue;
                };

                let finalizer_registry = realm.finalizer_registry;
                finalizer_registry.clear();
                drop(finalizer_registry);

                realm.context_shutdown_sig.done();
            }
        }

        if !worker_handle_state.worker_handle_active() && realms.len() == 0 {
            break;
        }
    }

    worker_shutdown_sig.done();

    Ok(())
}

#[allow(unused)]
#[rustfmt::skip]
impl std::fmt::Debug for JsWorkerEvent {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Self::CreateContext { resolve, context_shutdown_sig } =>        write!(f, "CreateContext"),
            Self::Exec { id, callback, span } =>                            write!(f, "Exec                     [id={}]", id),
            Self::BackgroundTaskComplete { id } =>                                                              write!(f, "BackgroundTaskComplete   [id={}]", id),
            Self::Import { id, specifier } =>                                                          write!(f, "Import"),
            Self::WorkerHandleDropped =>                                                                                write!(f, "WorkerHandleDropped"),
            Self::WorkerHandleDeactivated =>                                                                            write!(f, "WorkerHandleDeactivated"),
            Self::ContextHandleDropped { id } =>                                                                write!(f, "ContextHandleDropped"),
            Self::ContextHandleDeactivated { id } =>                                                            write!(f, "ContextHandleDeactivated [id={}]", id),
            Self::RunGarbageCollectionForTesting { resolve } =>                                            write!(f, "RunGarbageCollectionForTesting"),
        }
    }
}
