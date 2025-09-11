#[allow(non_camel_case_types)]
pub type __v8_context = *mut v8::Global<v8::Context>;

pub fn v8_new_context(
    isolate: *mut v8::Isolate,
    scope: &mut v8::HandleScope<'_, ()>,
) -> __v8_context {
    // Note: [`v8::Global::into_raw`] appears to have a memory leak
    let context_local = v8::Context::new(scope, Default::default());
    let context_global = v8::Global::new(unsafe { &mut *isolate }, context_local);
    let context = Box::into_raw(Box::new(context_global));
    context
}

pub fn v8_get_context(context: __v8_context) -> v8::Local<'static, v8::Context> {
    unsafe { *(context as *mut v8::Local<'static, v8::Context>) }
}

pub fn v8_get_context_address(context: __v8_context) -> usize {
    context as usize
}

pub fn v8_drop_context(context: __v8_context) -> v8::Global<v8::Context> {
    unsafe { *Box::from_raw(context) }
}
