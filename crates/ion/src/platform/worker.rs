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
use crate::utils::HashMapExt;
use crate::utils::PathExt;
use crate::utils::channel::oneshot;
use crate::utils::tokio_ext::LocalRuntimeExt;

use super::JsRealm;
use super::background_worker::BackgroundWorkerEvent;
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
    ShutdownContext {
        id: usize,
        resolve: Sender<()>,
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
    Shutdown {
        resolve: Sender<()>,
    },
    RunGarbageCollectionForTesting {
        resolve: Sender<()>,
    },
}

// Create a dedicated thread to host the isolate
pub(crate) fn start_js_worker_thread(
    tx_background: Sender<BackgroundWorkerEvent>,
    extensions: Vec<Arc<JsExtension>>,
    resolvers: Vec<DynResolver>,
) -> (Sender<JsWorkerEvent>, Mutex<Option<JoinHandle<()>>>) {
    let (tx, rx) = unbounded::<JsWorkerEvent>();

    let handle = thread::spawn({
        let tx: Sender<JsWorkerEvent> = tx.clone();
        move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .local_block_on(worker_thread_async(
                    tx,
                    rx,
                    tx_background,
                    extensions,
                    resolvers,
                ));
        }
    });

    (tx, Mutex::new(Some(handle)))
}

async fn worker_thread_async(
    tx: Sender<JsWorkerEvent>,
    rx: Receiver<JsWorkerEvent>,
    tx_background: Sender<BackgroundWorkerEvent>,
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

    while let Ok(event) = rx.recv_async().await {
        match event {
            JsWorkerEvent::CreateContext { resolve } => {
                let realm = JsRealm::new(
                    Rc::clone(&isolate),
                    fs.clone(),
                    resolvers.clone(),
                    tx_background.clone(),
                );
                let realm_id = realm.id();

                Extension::register_extensions(&realm, &extensions);

                realms.insert(realm_id.clone(), realm);
                resolve.try_send((realm_id, tx.clone()))?;
            }
            JsWorkerEvent::ShutdownContext { id, resolve } => {
                let realm = realms.try_remove(&id)?;
                active_context.set(&realm.context);
                let Some((context_scope, handle_scope)) = active_context.take() else {
                    panic!()
                };

                realm.notify_shutdown();
                realm.drain_async_tasks().await;

                drop(context_scope);
                drop(handle_scope);
                resolve.try_send(())?;
            }
            JsWorkerEvent::Exec { id, callback } => {
                let realm = realms.try_get(&id)?;
                active_context.set(&realm.context);

                if let Err(err) = callback(&realm.env()) {
                    // TODO global error handler
                    panic!("Callback errored {:?}", err)
                };
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
            JsWorkerEvent::Shutdown { resolve } => {
                for (_id, realm) in realms {
                    realm.notify_shutdown();
                    realm.drain_async_tasks().await;
                }
                resolve.try_send(())?;
                break;
            }
            JsWorkerEvent::RunGarbageCollectionForTesting { resolve } => {
                // isolate.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
                resolve.try_send(())?;
            }
        }
    }

    Ok(())
}
