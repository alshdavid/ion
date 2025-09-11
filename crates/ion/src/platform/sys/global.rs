use super::__v8_context_scope;
use super::v8_get_context;
use super::v8_get_context_scope;

use super::__v8_context;

#[allow(non_camel_case_types)]
pub type __v8_global_this = *mut v8::Global<v8::Object>;

pub fn v8_new_global_this(
    context: __v8_context,
    context_scope: __v8_context_scope,
) -> __v8_global_this {
    let scope = v8_get_context_scope(context_scope);

    // Note: [`v8::Global::into_raw`] appears to have a memory leak
    let global_local = v8_get_context(context).global(scope);
    let global_global = v8::Global::new(scope, global_local);
    Box::into_raw(Box::new(global_global))
}

pub fn v8_get_global_this(v8_global_this: __v8_global_this) -> v8::Local<'static, v8::Object> {
    unsafe { *(v8_global_this as *mut v8::Local<'static, v8::Object>) }
}

pub fn v8_drop_global_this(v8_global_this: __v8_global_this) -> v8::Global<v8::Object> {
    unsafe { *Box::from_raw(v8_global_this) }
}
