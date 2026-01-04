#[allow(non_camel_case_types)]
pub type __v8_root_scope = *mut v8::HandleScope<'static, ()>;

pub fn v8_new_root_scope(root_scope: v8::HandleScope<'_, ()>) -> __v8_root_scope {
    Box::into_raw(Box::new(root_scope)) as _
}

pub fn v8_get_root_scope(root_scope: __v8_root_scope) -> &'static mut v8::HandleScope<'static, ()> {
    unsafe { &mut *root_scope }
}

pub fn v8_drop_root_scope(root_scope: __v8_root_scope) -> v8::HandleScope<'static, ()> {
    unsafe { *Box::from_raw(root_scope) }
}
