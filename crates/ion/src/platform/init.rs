use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

static PLATFORM: OnceLock<()> = OnceLock::new();
static HAS_INIT: AtomicBool = AtomicBool::new(false);

/// The V8 Platform can only be initialized once per process
pub fn initialize_once_with_args(args: &[&str]) {
    PLATFORM.get_or_init(move || {
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
    });
}

pub fn initialize_once() {
    initialize_once_with_args(&[])
}

pub fn has_initialized() -> bool {
    HAS_INIT.load(Ordering::Acquire)
}
