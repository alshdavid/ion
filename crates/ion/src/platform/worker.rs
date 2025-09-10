#![allow(warnings)]
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;
use std::usize;

use flume::Receiver;
use flume::Sender;
use flume::bounded;
use flume::unbounded;
use tokio_util::task::TaskTracker;
use v8::Isolate;

use crate::DynResolver;
use crate::Env;
use crate::Error;
use crate::JsExtension;
use crate::ResolverContext;
use crate::fs::FileSystem;
use crate::platform::background_worker::BackgroundTaskManager;
use crate::utils::HashMapExt;
use crate::utils::PathExt;
use crate::utils::channel::oneshot;
use crate::utils::tokio_ext::LocalRuntimeExt;

use super::JsRealm;
use super::active_context::ActiveContext;
use super::extension::Extension;
use super::module::Module;
use super::module_map::ModuleMap;
use super::resolve::run_resolvers;
use super::v8::RawIsolate;

pub(crate) enum JsWorkerEvent {
    CreateContext {
        resolve: Sender<(usize, Sender<JsWorkerEvent>)>,
    },
    BackgroundTaskComplete {
        id: usize,
    },
    RequestContextShutdown {
        id: usize,
    },
    Exec {
        id: usize,
        callback: Box<dyn Send + FnOnce(&Env) -> crate::Result<()>>,
    },
    Import {
        id: usize,
        specifier: String,
        resolve: Sender<()>,
    },
    RequestShutdown {
        resolve: Sender<()>,
    },
    RunGarbageCollectionForTesting {
        resolve: Sender<()>,
    },
}

// Create a dedicated thread to host the isolate
pub(crate) fn start_js_worker_thread(
    background_task_manager: Arc<BackgroundTaskManager>,
    extensions: Vec<Arc<JsExtension>>,
    resolvers: Vec<DynResolver>,
) -> (
    Sender<JsWorkerEvent>,
    Mutex<Option<JoinHandle<crate::Result<()>>>>,
) {
    let (tx, rx) = unbounded::<JsWorkerEvent>();

    // Start a thread for the Isolate
    let handle: JoinHandle<crate::Result<()>> = thread::spawn({
        let tx: Sender<JsWorkerEvent> = tx.clone();
        move || worker_thread(tx, rx, background_task_manager, extensions, resolvers)
    });

    (tx, Mutex::new(Some(handle)))
}

fn worker_thread(
    tx: Sender<JsWorkerEvent>,
    rx: Receiver<JsWorkerEvent>,
    background_task_manager: Arc<BackgroundTaskManager>,
    extensions: Vec<Arc<JsExtension>>,
    resolvers: Vec<DynResolver>,
) -> crate::Result<()> {
    // One isolate per worker thread
    let isolate = RawIsolate::new(v8::Isolate::new(v8::CreateParams::default()));

    // Used to switch between context scopes on the same thread
    let mut active_context = ActiveContext::new(Rc::clone(&isolate));

    // Maintain a store of Global<Context> to help with cleanup on shutdown.
    let mut realms = HashMap::<usize, Box<JsRealm>>::new();
    let fs = FileSystem::Physical;

    let mut shutdown_requested = false;

    while let Ok(event) = rx.recv() {
        // println!("{:?}", event);
        match event {
            JsWorkerEvent::CreateContext { resolve } => {
                let realm = JsRealm::new(
                    Rc::clone(&isolate),
                    fs.clone(),
                    resolvers.clone(),
                    background_task_manager.clone(),
                    tx.clone(),
                );
                let realm_id = realm.id();
                active_context.set(&realm.context);

                Extension::register_extensions(&realm, &extensions);

                realms.insert(realm_id.clone(), realm);
                resolve.try_send((realm_id, tx.clone()))?;
            }
            JsWorkerEvent::RequestContextShutdown { id } => {
                // If there are async tasks pending then wait for them to complete
                {
                    let realm = realms.try_get_mut(&id)?;
                    let mut realm_shutdown_requested = realm.shutdown_requested.borrow_mut();
                    (*realm_shutdown_requested) = true;
                    if realm.global_refs.count() != 0 {
                        continue;
                    }
                };

                // If there are no async tasks then shutdown the context
                let realm = realms.try_remove(&id)?;
                active_context.set(&realm.context);
                let Some((context_scope, handle_scope)) = active_context.take() else {
                    panic!()
                };

                drop(context_scope);
                drop(handle_scope);

                if shutdown_requested && realms.is_empty() {
                    break;
                }
            }
            JsWorkerEvent::Exec { id, callback } => {
                let realm = realms.try_get(&id)?;
                active_context.set(&realm.context);

                if let Err(err) = callback(&realm.env()) {
                    // TODO global error handler
                    panic!("Callback errored {:?}", err)
                };
            }
            JsWorkerEvent::BackgroundTaskComplete { id } => {
                let realm = realms.try_get(&id)?;
                let mut realm_shutdown_requested = realm.shutdown_requested.borrow();
                if *realm_shutdown_requested && realm.global_refs.count() == 0 {
                    tx.try_send(JsWorkerEvent::RequestContextShutdown { id })?;
                }
            }
            JsWorkerEvent::Import {
                id,
                specifier,
                resolve,
            } => {
                let module = Module::v8_initialize(
                    true,
                    realms.try_get(&id)?,
                    &specifier,
                    std::env::current_dir()?.try_to_string()?,
                )?;

                let result = resolve.try_send(())?;
            }
            JsWorkerEvent::RequestShutdown { resolve } => {
                shutdown_requested = true;
                if realms.is_empty() {
                    break;
                }
            }
            JsWorkerEvent::RunGarbageCollectionForTesting { resolve } => {
                // isolate.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
                resolve.try_send(())?;
            }
        }
    }

    Ok(())
}

impl std::fmt::Debug for JsWorkerEvent {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Self::CreateContext { resolve } => write!(f, "CreateContext"),
            Self::BackgroundTaskComplete { id } => write!(f, "BackgroundTaskComplete"),
            Self::RequestContextShutdown { id } => write!(f, "RequestContextShutdown"),
            Self::Exec { id, callback } => write!(f, "Exec"),
            Self::Import {
                id,
                specifier,
                resolve,
            } => write!(f, "Import"),
            Self::RequestShutdown { resolve } => write!(f, "RequestShutdown"),
            Self::RunGarbageCollectionForTesting { resolve } => {
                write!(f, "RunGarbageCollectionForTesting")
            }
        }
    }
}
