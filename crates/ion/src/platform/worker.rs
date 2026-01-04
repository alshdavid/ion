use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
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
use crate::platform::callback_registry::CallbackRegistry;
use crate::utils::HashMapExt;
use crate::utils::PathExt;

pub(crate) enum JsWorkerEvent {
    CreateContext {
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
    TryShutdownContext {
        id: usize,
    },
    RegisterWorkerShutdownListener {
        resolve: Sender<()>,
    },
    RegisterContextShutdownListener {
        resolve: Option<Sender<()>>,
        id: usize,
    },
    RunGarbageCollectionForTesting {
        resolve: Sender<()>,
    },
    WorkerHandleDropped,
}

// Create a dedicated thread to host the isolate
#[allow(clippy::type_complexity)]
pub(crate) fn start_js_worker_thread(
    callback_registry: Arc<CallbackRegistry>,
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
                callback_registry,
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
    callback_registry: Arc<CallbackRegistry>,
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

    // Cleanup hooks
    let mut shutdown_context_senders = HashMap::<usize, Vec<Sender<()>>>::new();
    let mut shutdown_worker_senders = Vec::<Sender<()>>::new();
    let mut worker_handle_active = false;

    while let Ok(event) = rx.recv() {
        let shutdown_worker_senders = &mut shutdown_worker_senders;

        eprintln!("{:?}", event);
        match event {
            JsWorkerEvent::CreateContext { resolve } => {
                let realm = JsRealm::new(
                    isolate_ptr,
                    fs.clone(),
                    resolvers.clone(),
                    transformers.clone(),
                    background_task_manager.clone(),
                    tx.clone(),
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

                tx.try_send(JsWorkerEvent::TryShutdownContext { id }).unwrap();
            }
            JsWorkerEvent::Import { id, specifier } => {
                Module::v8_initialize(
                    true,
                    realms.try_get(&id)?,
                    &specifier,
                    std::env::current_dir()?.try_to_string()?,
                )?;
            }
            JsWorkerEvent::RegisterContextShutdownListener { id, resolve } => {
                // Store shutdown resolvers for when the context is closed
                if let Some(resolve) = resolve {
                    shutdown_context_senders
                        .entry(id)
                        .or_default()
                        .push(resolve);
                }
            }
            JsWorkerEvent::RegisterWorkerShutdownListener { resolve } => {
                shutdown_worker_senders.push(resolve);
            }
            JsWorkerEvent::RunGarbageCollectionForTesting { resolve } => {
                isolate.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
                resolve.try_send(())?;
            }
            JsWorkerEvent::WorkerHandleDropped => {
                worker_handle_active = false;
                for id in realms.keys() {
                    tx.try_send(JsWorkerEvent::TryShutdownContext { id: id.clone() }).unwrap();
                }
                if realms.len() == 0 {
                    break
                }
            }
            JsWorkerEvent::TryShutdownContext { id } => {
                println!("worker_handle_active: {}", worker_handle_active);
                if worker_handle_active {
                    continue;
                }

                // If there are async tasks pending then wait for them to complete
                {
                    let realm = realms.try_get_mut(&id)?;
                    if realm.global_refs.count() != 0 {
                        continue;
                    }
                };

                // If there are no async tasks then shutdown the context
                let Some(realm) = realms.remove(&id) else {
                    continue;
                };

                println!("ok");

                let finalizer_registry = realm.finalizer_registry;
                finalizer_registry.clear();
                drop(finalizer_registry);

                for resolver in shutdown_context_senders.remove(&id).unwrap_or_default() {
                    let _ = resolver.try_send(());
                }

                if realms.len() == 0 {
                    break
                }             
            }
        }
    }

    eprintln!("shutdown_worker_senders {}", shutdown_worker_senders.len());

    for sender in shutdown_worker_senders {
        let _ = sender.try_send(());
    }

    Ok(())
}

#[allow(unused)]
impl std::fmt::Debug for JsWorkerEvent {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Self::CreateContext { resolve } => write!(f, "CreateContext"),
            Self::Exec { id, callback, span } => write!(f, "Exec"),
            Self::Import { id, specifier } => write!(f, "Import"),
            Self::TryShutdownContext { id } => write!(f, "TryShutdownContext"),
            Self::RegisterContextShutdownListener { id, resolve } => write!(f, "RegisterContextShutdownListener"),
            Self::RegisterWorkerShutdownListener { resolve } => write!(f, "RegisterWorkerShutdownListener"),
            Self::RunGarbageCollectionForTesting { resolve } => {
                write!(f, "RunGarbageCollectionForTesting")
            }
            Self::WorkerHandleDropped => write!(f, "WorkerHandleDropped"),
        }
    }
}
