#[allow(non_camel_case_types)]
pub type __v8_context_scope = *mut v8::ContextScope<'static, v8::HandleScope<'static>>;

pub fn v8_new_context_scope(
    scope: v8::ContextScope<'static, v8::HandleScope<'static>>
) -> __v8_context_scope {
    Box::into_raw(Box::new(scope)) as _
}

pub fn v8_get_context_scope(
    context_scope: __v8_context_scope
) -> &'static mut v8::ContextScope<'static, v8::HandleScope<'static>> {
    unsafe { &mut *context_scope }
}

pub fn v8_drop_context_scope(
    context_scope: __v8_context_scope
) -> v8::ContextScope<'static, v8::HandleScope<'static>> {
    unsafe { *Box::from_raw(context_scope) }
}
