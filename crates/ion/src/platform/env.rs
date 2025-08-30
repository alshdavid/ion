use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::Rc;

use crate::event_loop::EventLoop;

use super::Error;

#[derive(Clone)]
pub struct Env {
    pub(super) isolate: *mut v8::Isolate,
    pub(super) handle_scope: *mut c_void,
    pub(super) context: *mut c_void,
    pub(super) context_scope: *mut c_void,
    pub(super) event_loop: Rc<RefCell<EventLoop>>, // pub(super) scope: *mut c_void,
}

impl std::fmt::Debug for Env {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("Env {}").finish()
    }
}

impl Env {
    pub fn isolate(&self) -> &mut v8::Isolate {
        unsafe { &mut *self.isolate }
    }

    pub fn context(&self) -> v8::Local<'static, v8::Context> {
        unsafe { *(self.context as *mut v8::Local<'static, v8::Context>) }
    }

    pub fn context_scope(&self) -> &mut v8::ContextScope<'static, v8::HandleScope<'static>> {
        unsafe {
            &mut *(self.context_scope as *mut v8::ContextScope<'static, v8::HandleScope<'static>>)
        }
    }

    pub fn open_scope<'a>(&'a self) -> v8::HandleScope<'a> {
        let v: v8::HandleScope<'a> = v8::HandleScope::new(self.context_scope());
        v
    }

    pub fn eval_script<S: AsRef<str>>(
        &self,
        code: S,
    ) -> crate::Result<v8::Local<'_, v8::Value>> {
        let scope = &mut self.open_scope();
        let Some(code) = v8::String::new(scope, code.as_ref()) else {
            return Err(Error::StringCreateError);
        };
        let Some(script) = v8::Script::compile(scope, code, None) else {
            return Err(Error::ScriptCompileError);
        };
        let Some(value) = script.run(scope) else {
            return Err(Error::ScriptRunError);
        };
        Ok(value)
    }

    pub fn spawn_async(
        &self,
        task: impl Future<Output = ()> + 'static,
    ) -> crate::Result<()> {
        let event_loop = self.event_loop.borrow();
        event_loop.spawn_local(task)
    }
}
