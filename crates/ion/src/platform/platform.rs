use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;

use flume::Sender;
use flume::unbounded;

use super::JsWorker;

pub(crate) static HAS_INIT: AtomicBool = AtomicBool::new(false);

pub(crate) static PLATFORM: LazyLock<Sender<PlatformEvent>> = LazyLock::new(|| {
    let (tx, rx) = unbounded();

    // Dedicated thread for the v8 platform
    // All Isolates need to be in this thread or in children threads of this thread
    thread::spawn(move || {
        while let Ok(event) = rx.recv() {
            match event {
                PlatformEvent::Init(args) => {
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
                PlatformEvent::SpawnWorker(tx) => {
                    if tx.send(Arc::new(JsWorker::new())).is_err() {
                        // TODO implement global error handler
                        panic!("Internal error starting worker")
                    };
                }
            }
        }
    });

    tx
});

pub(crate) enum PlatformEvent {
    Init(Vec<String>),
    SpawnWorker(Sender<Arc<JsWorker>>),
}
