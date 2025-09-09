use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use flume::Sender;
use tokio_util::task::TaskTracker;

use crate::FromJsValue;
use crate::JsObject;
use crate::platform::JsRealm;
use crate::platform::Value;
use crate::platform::background_worker::BackgroundWorkerEvent;
use crate::platform::module::Module;
use crate::platform::v8::RawContext;
use crate::platform::v8::RawGlobal;
use crate::platform::v8::RawIsolate;
use crate::utils::generate_random_string;

#[derive(Clone)]
pub struct Env {
    pub(crate) inner: *mut Env,
    pub(crate) isolate: Rc<RawIsolate>,
    pub(crate) context: Rc<RawContext>,
    pub(crate) global_this: Rc<RawGlobal>,
    pub(crate) on_before_exit: RefCell<Vec<Rc<dyn 'static + Fn() -> crate::Result<()>>>>,
    pub(crate) shutdown_has_run: RefCell<bool>,
    // TODO make these refcells
    pub(crate) async_tasks: *mut TaskTracker,
    pub(crate) background_tasks: *mut Sender<BackgroundWorkerEvent>,
}

impl Env {
    pub(crate) fn new(
        isolate: Rc<RawIsolate>,
        context: Rc<RawContext>,
        global_this: Rc<RawGlobal>,
        async_tasks: *mut TaskTracker,
        background_tasks: *mut Sender<BackgroundWorkerEvent>,
    ) -> Box<Self> {
        let on_before_exit =
            RefCell::new(Vec::<Rc<dyn 'static + Fn() -> crate::Result<()>>>::new());

        let shutdown_has_run = RefCell::new(false);

        let mut env = Box::new(Env {
            isolate,
            context,
            global_this,
            async_tasks,
            background_tasks,
            inner: std::ptr::null_mut(),
            on_before_exit,
            shutdown_has_run,
        });

        env.inner = env.as_mut() as *mut Env;
        env
    }

    pub fn into_raw(&self) -> *mut Env {
        self.inner
    }

    /// # SAFETY
    ///
    /// Env only lives for as long as the v8::Context
    pub unsafe fn from_raw(r: *mut Env) -> Env {
        unsafe { (*r).clone() }
    }

    pub(crate) fn async_tasks(&self) -> &TaskTracker {
        unsafe { &mut *self.async_tasks }
    }

    pub(crate) fn background_tasks(&self) -> &Sender<BackgroundWorkerEvent> {
        unsafe { &mut *self.background_tasks }
    }

    pub fn isolate(&mut self) -> &mut v8::Isolate {
        // SAFETY: Lifetime of `Isolate` is longer than `Env`.
        self.isolate.as_mut()
    }

    pub fn global_this(&self) -> crate::Result<JsObject> {
        let v = self.global_this.as_inner();
        JsObject::from_js_value(self, Value::from(v.cast()))
    }

    pub fn context(&self) -> v8::Local<'static, v8::Context> {
        self.context.as_inner()
    }

    pub fn scope(&self) -> v8::CallbackScope<'static> {
        let context = self.context.as_inner();
        unsafe { v8::CallbackScope::new(context) }
    }

    /// Non blocking action on the current thread.
    /// Note: [`v8::HandleScope`]s don't survive a call to ".await"
    pub fn spawn_local(
        &self,
        fut: impl 'static + Future<Output = crate::Result<()>>,
    ) -> crate::Result<()> {
        self.async_tasks().spawn_local(fut);
        Ok(())
    }

    pub fn on_before_exit(
        &self,
        callback: impl 'static + Fn() -> crate::Result<()>,
    ) {
        let mut shutdown_has_run = self.shutdown_has_run.borrow_mut();
        let mut on_before_exit = self.on_before_exit.borrow_mut();
        (*shutdown_has_run) = true;
        on_before_exit.push(Rc::new(callback));
    }

    pub fn shutdown_has_run(&self) -> bool {
        let shutdown_has_run = self.shutdown_has_run.borrow();
        shutdown_has_run.clone()
    }

    /// Send a task to a background thread
    pub fn spawn_background(
        &self,
        fut: impl 'static + Send + Sync + Future<Output = crate::Result<()>>,
    ) -> crate::Result<()> {
        self.background_tasks()
            .try_send(BackgroundWorkerEvent::ExecFut(Box::pin(fut)))?;
        Ok(())
    }

    pub fn eval_script<Return: FromJsValue>(
        &self,
        code: impl AsRef<str>,
    ) -> crate::Result<Return> {
        let scope = &mut self.scope();

        let Some(code) = v8::String::new(scope, code.as_ref()) else {
            panic!();
        };

        let Some(script) = v8::Script::compile(scope, code, None) else {
            panic!();
        };

        let Some(value) = script.run(scope) else {
            panic!();
        };

        Return::from_js_value(self, Value::from(value))
    }

    pub fn eval_module(
        &self,
        code: impl AsRef<str>,
    ) -> crate::Result<JsObject> {
        let scope = &mut self.scope();
        let realm = JsRealm::v8_revive(scope);

        let module = Module::new(realm, generate_random_string(20), code.as_ref())?;

        let v8_module = Module::v8_run_module(true, realm, module.name.clone(), module)?;
        let v8_module = v8_module.get_module_namespace().cast::<v8::Object>();

        JsObject::from_js_value(self, Value::from(v8_module.cast()))
    }

    /// Load a file and evaluate it
    pub fn import(
        &self,
        _path: impl AsRef<Path>,
    ) -> crate::Result<()> {
        todo!()
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        // drop(unsafe { Box::from_raw(self.on_before_exit)});
        // drop(unsafe { Box::from_raw(self.shutdown_has_run) });
    }
}
