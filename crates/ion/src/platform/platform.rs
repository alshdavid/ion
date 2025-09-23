use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;
use std::thread::JoinHandle;

use flume::Sender;
use flume::unbounded;

use crate::JsExtension;
use crate::JsResolver;
use crate::JsTransformer;
use crate::platform::background_worker::BackgroundTaskManager;
use crate::platform::worker::JsWorkerEvent;
use crate::platform::worker::start_js_worker_thread;

pub(crate) enum PlatformEvent {
    Init {
        skip_init: bool,
        args: Vec<String>,
        extensions: Vec<JsExtension>,
        resolvers: Vec<JsResolver>,
        transformers: Vec<JsTransformer>,
    },
    SpawnWorker {
        extensions: Vec<JsExtension>,
        resolvers: Vec<JsResolver>,
        transformers: Vec<JsTransformer>,
        #[allow(clippy::type_complexity)]
        resolve: Sender<(
            Sender<JsWorkerEvent>,
            Mutex<Option<JoinHandle<crate::Result<()>>>>,
        )>,
    },
}

pub(crate) static HAS_INIT: AtomicBool = AtomicBool::new(false);

pub(crate) static PLATFORM: LazyLock<Sender<PlatformEvent>> = LazyLock::new(|| {
    let (tx, rx) = unbounded();

    // Dedicated thread for the v8 platform
    // All Isolates need to be in this thread or in children threads of this thread
    thread::spawn(move || {
        let background_task_manager = Arc::new(BackgroundTaskManager::new().unwrap());

        let mut extensions = Vec::<Arc<JsExtension>>::new();
        let mut resolvers = Vec::<JsResolver>::new();
        let mut transformers = HashMap::<String, Arc<JsTransformer>>::new();

        transformers.insert("ts".to_string(), Arc::new(crate::transformers::ts()));
        transformers.insert("tsx".to_string(), Arc::new(crate::transformers::tsx()));

        while let Ok(event) = rx.recv() {
            match event {
                PlatformEvent::Init {
                    skip_init,
                    args,
                    extensions: init_extensions,
                    resolvers: init_resolvers,
                    transformers: init_transformers,
                } => {
                    if !skip_init {
                        let platform = v8::new_default_platform(0, false).make_shared();

                        if !args.is_empty() {
                            let args = args
                                .iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<String>>()
                                .join(" ");

                            v8::V8::set_flags_from_string(&args);
                        }

                        v8::V8::initialize_platform(platform);
                        v8::V8::initialize();
                    }

                    HAS_INIT.store(true, Ordering::Release);

                    for extension in init_extensions {
                        extensions.push(Arc::new(extension))
                    }

                    for resolver in init_resolvers {
                        resolvers.push(resolver)
                    }

                    for transformer in init_transformers {
                        transformers.insert(transformer.kind.clone(), Arc::new(transformer));
                    }
                }
                PlatformEvent::SpawnWorker {
                    resolve,
                    extensions: init_extensions,
                    resolvers: init_resolvers,
                    transformers: init_transformers,
                } => {
                    let mut worker_extensions = extensions.clone();
                    for extension in init_extensions {
                        worker_extensions.push(Arc::new(extension))
                    }

                    let mut worker_resolvers = resolvers.clone();
                    for resolver in init_resolvers {
                        worker_resolvers.push(resolver)
                    }

                    let mut worker_transformers = transformers.clone();
                    for transformer in init_transformers {
                        worker_transformers.insert(transformer.kind.clone(), Arc::new(transformer));
                    }

                    let (tx, handle) = start_js_worker_thread(
                        background_task_manager.clone(),
                        worker_extensions,
                        worker_resolvers,
                        worker_transformers,
                    );

                    if resolve.try_send((tx, handle)).is_err() {
                        // TODO implement global error handler
                        panic!("Internal error starting worker")
                    };
                }
            }
        }
    });

    tx
});
