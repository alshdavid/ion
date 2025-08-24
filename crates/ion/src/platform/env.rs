use std::ptr::NonNull;

pub struct Env {
    pub(crate) isolate_ptr: *mut v8::Isolate,
    pub(crate) inner_context: NonNull<v8::Context>,
}

impl Env {
    pub(crate) fn isolate(&self) -> &mut v8::Isolate {
        // SAFETY: This is valid for the lifetime of `JsRuntime` that created this env
        unsafe { &mut *self.isolate_ptr }
    }

    pub(crate) fn context(&self) -> v8::Local<'static, v8::Context> {
        // SAFETY: `v8::Local` is always non-null pointer; the `HandleScope` is
        // already on the stack, but we don't have access to it.
        unsafe {
            std::mem::transmute::<NonNull<v8::Context>, v8::Local<v8::Context>>(self.inner_context)
        }
    }

    pub fn scope(&self) -> v8::HandleScope<'_> {
        v8::HandleScope::with_context(self.isolate(), self.context())
    }
}
