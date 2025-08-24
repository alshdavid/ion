use std::ptr::NonNull;

use crate::Env;

use super::FromJsValue;

pub struct JsRuntime {
    _inner_isolate: v8::OwnedIsolate,
    inner_isolate_ptr: *mut v8::Isolate,
    inner_context: NonNull<v8::Context>,
}

impl JsRuntime {
    pub fn new() -> Self {
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());
        let isolate_ptr = isolate.as_mut() as *mut v8::Isolate;

        let context = {
            let mut main_scope = v8::HandleScope::new(&mut isolate);
            let context = v8::Context::new(&mut main_scope, Default::default());
            let global_context = v8::Global::new(&mut main_scope, context);
            global_context
        };

        JsRuntime {
            _inner_isolate: isolate,
            inner_isolate_ptr: isolate_ptr,
            inner_context: context.into_raw(),
        }
    }

    pub fn eval<'a, R: FromJsValue<v8::Local<'a, v8::Value>>>(
        &'a self,
        code: &str,
    ) -> R {
        let mut scope = self.scope();
        let code = v8::String::new(&mut scope, code.as_ref()).unwrap();
        let script = v8::Script::compile(&mut scope, code, None).unwrap();
        let result = script.run(&mut scope).unwrap();
        R::from_js_value(&mut scope, result)
    }

    pub fn exec<R>(
        &self,
        callback: impl FnOnce(Env) -> R,
    ) -> R {
        callback(Env {
            isolate_ptr: self.inner_isolate_ptr,
            inner_context: self.inner_context,
        })
    }

    fn isolate(&self) -> &mut v8::Isolate {
        // SAFETY: This is valid for the lifetime of the struct
        unsafe { &mut *self.inner_isolate_ptr }
    }

    fn context(&self) -> v8::Local<'static, v8::Context> {
        // SAFETY: `v8::Local` is always non-null pointer; the `HandleScope` is
        // already on the stack, but we don't have access to it.
        unsafe {
            std::mem::transmute::<NonNull<v8::Context>, v8::Local<v8::Context>>(self.inner_context)
        }
    }

    fn scope(&self) -> v8::HandleScope<'_> {
        v8::HandleScope::with_context(self.isolate(), self.context())
    }
}
