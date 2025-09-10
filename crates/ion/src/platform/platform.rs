use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;
use std::thread::JoinHandle;

use flume::Sender;
use flume::unbounded;

use crate::DynResolver;
use crate::JsExtension;
use crate::platform::background_worker::BackgroundTaskManager;
use crate::platform::worker::JsWorkerEvent;
use crate::platform::worker::start_js_worker_thread;

pub(crate) enum PlatformEvent {
    Init {
        args: Vec<String>,
    },
    SpawnWorker {
        #[allow(clippy::type_complexity)]
        resolve: Sender<(
            Sender<JsWorkerEvent>,
            Mutex<Option<JoinHandle<crate::Result<()>>>>,
        )>,
    },
    RegisterExtension(JsExtension, Sender<crate::Result<()>>),
    RegisterResolver {
        resolver: DynResolver,
        resolve: Sender<crate::Result<()>>,
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
        let mut resolvers = Vec::<DynResolver>::new();

        while let Ok(event) = rx.recv() {
            match event {
                PlatformEvent::Init { args } => {
                    let platform = v8::new_default_platform(0, false).make_shared();

                    if !args.is_empty() {
                        // Debug args
                        // "--no_freeze_flags_after_init --expose_gc --harmony-shadow-realm --allow_natives_syntax --turbo_fast_api_calls --js-source-phase-imports",
                        let args = args
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<String>>()
                            .join(" ");

                        v8::V8::set_flags_from_string(&args);
                    }

                    v8::V8::initialize_platform(platform);
                    v8::V8::initialize();

                    HAS_INIT.store(true, Ordering::Release);
                }
                PlatformEvent::SpawnWorker { resolve } => {
                    let (tx, handle) = start_js_worker_thread(
                        background_task_manager.clone(),
                        extensions.clone(),
                        resolvers.clone(),
                    );

                    if resolve.try_send((tx, handle)).is_err() {
                        // TODO implement global error handler
                        panic!("Internal error starting worker")
                    };
                }
                PlatformEvent::RegisterExtension(extension, tx_reply) => {
                    extensions.push(Arc::new(extension));
                    tx_reply.try_send(Ok(())).unwrap();
                }
                PlatformEvent::RegisterResolver { resolver, resolve } => {
                    resolvers.push(resolver);
                    resolve.try_send(Ok(())).unwrap();
                }
            }
        }
    });

    tx
});
