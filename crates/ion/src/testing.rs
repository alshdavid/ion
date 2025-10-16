use std::sync::Arc;
use std::sync::LazyLock;

use crate::*;

pub static JS_RUNTIME: LazyLock<Arc<JsRuntime>> = LazyLock::new(|| {
    JsRuntime::initialize_once(JsRuntimeOptions::debug(JsRuntimeOptions {
        v8_args: vec![],
        resolvers: vec![],
        transformers: vec![],
        extensions: vec![],
    }))
    .expect("Unable to start runtime")
});
