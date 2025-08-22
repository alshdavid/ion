use std::sync::OnceLock;
use std::sync::atomic::Ordering;

use super::JsRuntime;
use super::platform::HAS_INIT;
use super::platform::PLATFORM;
use super::platform::PlatformEvent;

static RUNTIME: OnceLock<super::Result<JsRuntime>> = OnceLock::new();

/// Initialize the v8 runtime, this can only be done once per process.
/// Subsequent calls will return the first instance of [`JsRuntime`]
pub fn initialize_once_with_args(args: &[&str]) -> super::Result<JsRuntime> {
    match RUNTIME.get_or_init(|| {
        let args = args.iter().map(|v| v.to_string()).collect::<Vec<String>>();

        if PLATFORM.send(PlatformEvent::Init(args)).is_err() {
            return Err(super::Error::PlatformInitializeError);
        };

        let rt = JsRuntime {
            tx: PLATFORM.clone(),
        };

        Ok(rt)
    }) {
        Ok(rt) => Ok(rt.clone()),
        Err(err) => Err(err.clone()),
    }
}

/// Initialize the v8 runtime, this can only be done once per process.
/// Subsequent calls will return the first instance of [`JsRuntime`]
pub fn initialize_once() -> super::Result<JsRuntime> {
    initialize_once_with_args(&[])
}

/// Check if the v8 runtime has already been initialized
pub fn has_initialized() -> bool {
    HAS_INIT.load(Ordering::Acquire)
}
