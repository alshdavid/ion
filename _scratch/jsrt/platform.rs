use std::sync::OnceLock;

/// Initialize v8 platform once and persist it until the process exits.
/// This can only be done once. Currently this is not configurable
pub fn platform_once_init() {
  static PLATFORM_INIT: OnceLock<()> = OnceLock::new();

  PLATFORM_INIT.get_or_init(|| {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();
  });
}

