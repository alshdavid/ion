use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::Ordering;

use flume::Sender;
use flume::bounded;

use super::platform::platform::PlatformEvent;
use crate::Error;
use crate::JsExtension;
use crate::JsResolver;
use crate::JsTransformer;
use crate::JsWorker;
use crate::JsWorkerOptions;
use crate::platform::platform::HAS_INIT;
use crate::platform::platform::PLATFORM;

static JS_RUNTIME: OnceLock<crate::Result<Arc<JsRuntime>>> = OnceLock::new();

#[derive(Default)]
pub struct JsRuntimeOptions {
    /// Arguments to pass through to v8
    pub v8_args: Vec<String>,
    /// Hook that runs before code is imported. This can be used to
    /// customize the behavior of "import" statements
    pub resolvers: Vec<JsResolver>,
    /// Hook that runs before code is loaded. This can be used to
    /// convert TypeScript into JavaScript or JSON into JavaScript
    pub transformers: Vec<JsTransformer>,
    /// Extensions that will be available to all [`crate::JsWorker`] and [`crate::JsContext`] instances
    pub extensions: Vec<JsExtension>,
}

impl JsRuntimeOptions {
    /// Apply debugging arguments to options
    pub fn debug(options: JsRuntimeOptions) -> Self {
        let mut v8_args = vec![
            "--no_freeze_flags_after_init".to_string(),
            "--expose_gc".to_string(),
            "--harmony-shadow-realm".to_string(),
            "--allow_natives_syntax".to_string(),
            "--turbo_fast_api_calls".to_string(),
            "--js-source-phase-imports".to_string(),
        ];

        v8_args.extend(options.v8_args);

        JsRuntimeOptions {
            v8_args,
            extensions: options.extensions,
            transformers: options.transformers,
            resolvers: options.resolvers,
        }
    }
}

/// JsRuntime is a handle to the underlying v8 engine. It can spawn worker threads and
/// evaluate JavaScript within Contexts
#[derive(Clone, Debug)]
pub struct JsRuntime {
    pub(crate) tx: Sender<crate::platform::platform::PlatformEvent>,
}

impl JsRuntime {
    /// Method to allow initialisation of V8 on the current thread,
    /// rather than the dedicated Ion platform thread. Allows Ion to be used
    /// alongside other runtimes/isolate management on x86 platforms.
    pub fn initialize_local(options: JsRuntimeOptions) -> crate::Result<Arc<JsRuntime>> {
        let platform = v8::new_default_platform(0, false).make_shared();

        if !options.v8_args.is_empty() {
            let args = options
                .v8_args
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(" ");

            v8::V8::set_flags_from_string(&args);
        }

        v8::V8::initialize_platform(platform);
        v8::V8::initialize();

        // Mark as initialized to prevent double initialization
        crate::platform::platform::HAS_INIT.store(true, std::sync::atomic::Ordering::Release);

        // Return the runtime instance, same as regular initialization
        JS_RUNTIME
            .get_or_init(move || {
                let args = options
                    .v8_args
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>();

                if PLATFORM
                    .send(PlatformEvent::Init {
                        skip_init: true,
                        args,
                        extensions: options.extensions,
                        transformers: options.transformers,
                        resolvers: options.resolvers,
                    })
                    .is_err()
                {
                    return Err(crate::Error::PlatformInitializeError);
                };

                Ok(Arc::new(JsRuntime {
                    tx: PLATFORM.clone(),
                }))
            })
            .clone()
    }

    /// Initialize the v8 runtime, this can only be done once per process.
    /// Subsequent calls will return the first instance of [`JsRuntime`]
    pub fn initialize_once(options: JsRuntimeOptions) -> crate::Result<Arc<JsRuntime>> {
        JS_RUNTIME
            .get_or_init(move || {
                let args = options
                    .v8_args
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>();

                if PLATFORM
                    .send(PlatformEvent::Init {
                        skip_init: false,
                        args,
                        extensions: options.extensions,
                        transformers: options.transformers,
                        resolvers: options.resolvers,
                    })
                    .is_err()
                {
                    return Err(crate::Error::PlatformInitializeError);
                };

                Ok(Arc::new(JsRuntime {
                    tx: PLATFORM.clone(),
                }))
            })
            .clone()
    }

    /// Check if the v8 runtime has already been initialized
    pub fn has_initialized() -> bool {
        HAS_INIT.load(Ordering::Acquire)
    }

    /// Spawns a dedicated worker thread for isolates
    pub fn spawn_worker(
        &self,
        options: JsWorkerOptions,
    ) -> crate::Result<Arc<JsWorker>> {
        let (tx, rx) = bounded(1);

        if self
            .tx
            .send(PlatformEvent::SpawnWorker {
                extensions: options.extensions,
                transformers: options.transformers,
                resolvers: options.resolvers,
                resolve: tx,
            })
            .is_err()
        {
            return Err(Error::WorkerInitializeError);
        };

        let Ok((tx, handle)) = rx.recv() else {
            return Err(Error::WorkerInitializeError);
        };

        Ok(Arc::new(JsWorker::new(tx, handle)))
    }
}

impl Drop for JsRuntime {
    fn drop(&mut self) {
        // Once initialized, JsRuntime is never dropped
        // Only the worker threads and isolates are dropped
    }
}
