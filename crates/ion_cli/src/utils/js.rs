use std::ptr::NonNull;


pub fn v8_set_obj_property<'a, S: AsRef<str>>(
    scope: &mut v8::HandleScope<'a>,
    object: &v8::Local<'a, v8::Object>,
    key: S,
    value: v8::Local<'a, v8::Value>,
) {
    let key = v8::String::new(scope, key.as_ref()).unwrap();
    object.define_property(
        scope,
        key.into(),
        &v8::PropertyDescriptor::new_from_value(value),
    );
}

pub fn v8_get_obj_property<'a, S: AsRef<str>>(
    scope: &mut v8::HandleScope<'a>,
    object: &v8::Local<'a, v8::Object>,
    key: S,
) -> Option<v8::Local<'a, v8::Value>> {
    let key = v8::String::new(scope, key.as_ref()).unwrap();
    object.get(scope, key.into())
}

pub fn v8_delete_obj_property<'a, S: AsRef<str>>(
    scope: &mut v8::HandleScope<'a>,
    object: &v8::Local<'a, v8::Object>,
    key: S,
) {
    let key = v8::String::new(scope, key.as_ref()).unwrap();
    object.delete(scope, key.into());
}

pub fn v8_eval_code<'a, S: AsRef<str>>(
    scope: &mut v8::HandleScope<'a>,
    code: S,
) -> v8::Global<v8::Value> {
    let code = v8::String::new(scope, code.as_ref()).unwrap();
    let script = v8::Script::compile(scope, code, None).unwrap();
    let value = script.run(scope).unwrap();
    v8::Global::new(scope, value)
}

pub fn v8_global_context(context_ptr: NonNull<v8::Context>) -> v8::Local<'static, v8::Context> {
    // SAFETY: `v8::Local` is always non-null pointer; the `HandleScope` is
    // already on the stack, but we don't have access to it.
    unsafe { std::mem::transmute::<NonNull<v8::Context>, v8::Local<v8::Context>>(context_ptr) }
}

pub fn v8_global_scope(
    isolate_ptr: *mut v8::Isolate,
    context_ptr: NonNull<v8::Context>,
) -> v8::HandleScope<'static> {
    v8::HandleScope::with_context(v8_isolate_ptr(isolate_ptr), v8_global_context(context_ptr))
}

pub fn v8_isolate_ptr(isolate_ptr: *mut v8::Isolate) -> &'static mut v8::Isolate {
    unsafe { &mut *isolate_ptr }
}
