use std::ffi::c_void;

use super::Error;

#[derive(Debug, Clone)]
pub struct Env {
    pub(super) isolate: *mut v8::Isolate,
    pub(super) handle_scope: *mut c_void,
    pub(super) context: *mut c_void,
    pub(super) context_scope: *mut c_void,
    pub(super) scope: *mut c_void,
}

impl Env {
    pub(crate) fn isolate(&self) -> &mut v8::Isolate {
        unsafe { &mut *self.isolate }
    }

    pub(crate) fn context(&self) -> v8::Local<'static, v8::Context> {
        unsafe {
            let v = self.context as *mut v8::Global<v8::Context>;
            std::mem::transmute::<*mut v8::Global<v8::Context>, v8::Local<v8::Context>>(v)
        }
    }

    pub(crate) fn context_scope(&self) -> &mut v8::ContextScope<'static, v8::HandleScope<'static>> {
        unsafe {
            &mut *(self.context_scope as *mut v8::ContextScope<'static, v8::HandleScope<'static>>)
        }
    }

    pub fn scope(&self) -> &mut v8::HandleScope<'static> {
        unsafe { &mut *(self.scope as *mut v8::HandleScope<'static>) }
    }

    pub fn eval_script<S: AsRef<str>>(&self, code: S) -> crate::Result<v8::Local<'_, v8::Value>> {
        let Some(code) = v8::String::new(self.scope(), code.as_ref()) else {
            return Err(Error::StringCreateError);
        };
        let Some(script) = v8::Script::compile(self.scope(), code, None) else {
            return Err(Error::ScriptCompileError);
        };
        let Some(value) = script.run(self.scope()) else {
            return Err(Error::ScriptRunError);
        };
        Ok(value)
    }
}
