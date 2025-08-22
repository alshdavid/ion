use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::Rc;
use std::time::Duration;

use super::Error;

#[derive(Clone)]
pub struct Env {
    pub(super) isolate: *mut v8::Isolate,
    pub(super) handle_scope: *mut c_void,
    pub(super) context: *mut c_void,
    pub(super) context_scope: *mut c_void,
    pub(super) tasks: Rc<RefCell<Option<tokio::task::LocalSet>>>,
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
    pub(crate) fn new(isolate: *mut v8::Isolate) -> Self {
        let handle_scope = v8::HandleScope::new(unsafe { &mut *isolate });

        let handle_scope_ptr = Box::new(handle_scope);
        let handle_scope_ptr = Box::into_raw(handle_scope_ptr);

        let context = v8::Context::new(unsafe { &mut *handle_scope_ptr }, Default::default());
        let mut context_scope = v8::ContextScope::new(unsafe { &mut *handle_scope_ptr }, context);

        let global_context = v8::Global::new(&mut context_scope, context);
        let global_context_ptr = Box::into_raw(Box::new(global_context));

        let context_scope_ptr = Box::new(context_scope);
        let context_scope_ptr = Box::into_raw(context_scope_ptr);

        Self {
            isolate,
            handle_scope: handle_scope_ptr as _,
            context: global_context_ptr as _,
            context_scope: context_scope_ptr as _,
            tasks: Default::default(),
        }
    }

    pub fn id(&self) -> usize {
        self.context as _
    }

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

    pub fn global_this<'a>(&'a self) -> v8::Local<'a, v8::Object> {
        let context = self.context();
        let scope = self.context_scope();
        context.global(scope)
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

    pub fn spawn_async_local(
        &self,
        task: impl Future<Output = ()> + 'static,
    ) -> crate::Result<()> {
        let mut set = self.tasks.borrow_mut();
        if set.is_none() {
            set.replace(Default::default());
        }
        let Some(set) = set.as_mut() else { panic!() };
        set.spawn_local(task);
        Ok(())
    }

    pub fn sleep(
        &self,
        duration: Duration,
    ) -> tokio::time::Sleep {
        tokio::time::sleep(duration)
    }
}
