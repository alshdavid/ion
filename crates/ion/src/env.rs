use std::cell::RefCell;
use std::path::Path;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

use flume::Sender;

use crate::AsyncEnv;
use crate::FromJsValue;
use crate::JsObject;
use crate::platform::JsRealm;
use crate::platform::Value;
use crate::platform::background_worker::BackgroundTaskManager;
use crate::platform::module::Module;
use crate::platform::v8::RawContext;
use crate::platform::v8::RawGlobal;
use crate::platform::v8::RawIsolate;
use crate::platform::worker::JsWorkerEvent;
use crate::utils::RefCounter;
use crate::utils::generate_random_string;

#[derive(Clone)]
pub struct Env {
    pub(crate) inner: *mut Env,
    pub(crate) realm_id: usize,
    pub(crate) isolate: Rc<RawIsolate>,
    pub(crate) context: Rc<RawContext>,
    pub(crate) global_this: Rc<RawGlobal>,
    pub(crate) background_task_manager: Arc<BackgroundTaskManager>,
    pub(crate) global_refs: RefCounter,
    pub(crate) shutdown_requested: Rc<RefCell<bool>>,
    pub(crate) tx: Sender<JsWorkerEvent>,

    // Delete
    pub(crate) on_before_exit: RefCell<Vec<Rc<dyn 'static + Fn() -> crate::Result<()>>>>,
    pub(crate) shutdown_has_run: RefCell<bool>,
}

impl Env {
    pub(crate) fn new(
        isolate: Rc<RawIsolate>,
        context: Rc<RawContext>,
        global_this: Rc<RawGlobal>,
        background_task_manager: Arc<BackgroundTaskManager>,
        global_refs: RefCounter,
        shutdown_requested: Rc<RefCell<bool>>,
        tx: Sender<JsWorkerEvent>,
    ) -> Box<Self> {
        let on_before_exit =
            RefCell::new(Vec::<Rc<dyn 'static + Fn() -> crate::Result<()>>>::new());

        let shutdown_has_run = RefCell::new(false);

        let mut env = Box::new(Env {
            realm_id: 0,
            isolate,
            context,
            global_this,
            background_task_manager,
            inner: std::ptr::null_mut(),
            on_before_exit,
            shutdown_has_run,
            global_refs,
            shutdown_requested,
            tx,
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

    /// Incrementing the ref count prevents the context
    /// associated with this Env from being destroyed.
    ///
    /// A ref count of 0 means it's safe to shutdown the context.
    /// This is used for keeping the context open when there are
    /// async tasks running in the background.
    pub fn inc_ref(&self) {
        self.global_refs.inc();
    }

    pub fn dec_ref(&self) {
        self.global_refs.dec();
        let shutdown_requested = self.shutdown_requested.borrow();
        if self.global_refs.count() == 0 && *shutdown_requested {
            self.tx
                .try_send(JsWorkerEvent::RequestContextShutdown { id: self.realm_id })
                .unwrap();
        }
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

    pub fn on_before_exit(&self, callback: impl 'static + Fn() -> crate::Result<()>) {
        let mut shutdown_has_run = self.shutdown_has_run.borrow_mut();
        let mut on_before_exit = self.on_before_exit.borrow_mut();
        (*shutdown_has_run) = true;
        on_before_exit.push(Rc::new(callback));
    }

    pub fn shutdown_has_run(&self) -> bool {
        let shutdown_has_run = self.shutdown_has_run.borrow();
        shutdown_has_run.clone()
    }

    pub fn spawn_background(
        &self,
        callback: impl 'static
        + Send
        + Sync
        + FnOnce(
            AsyncEnv,
        ) -> Pin<
            Box<dyn 'static + Send + Sync + Future<Output = crate::Result<()>>>,
        >,
    ) -> crate::Result<()> {
        let async_env = AsyncEnv {
            tx: self.tx.clone(),
            realm_id: self.realm_id,
        };

        self.background_task_manager.spawn(async move {
            callback(async_env).await?;
            Ok(())
        })
    }

    /// Send a task to a background thread
    pub fn spawn_background_then<Result: 'static + Send + Sync>(
        &self,
        fut: impl 'static + Send + Sync + Future<Output = crate::Result<Result>>,
        then_fn: impl 'static + FnOnce(&Env, Result) -> crate::Result<()>,
    ) -> crate::Result<()> {
        let tx = self.tx.clone();
        let realm_id = self.realm_id;

        let then_fn: Box<dyn 'static + FnOnce(&Env, Result) -> crate::Result<()>> =
            Box::new(then_fn);
        let then_fn = Box::into_raw(Box::new(then_fn));
        let then_fn = then_fn as usize;

        self.background_task_manager.spawn(async move {
            let result = match fut.await {
                Ok(result) => result,
                Err(_) => {
                    let then_fn = then_fn
                        as *mut Box<dyn 'static + FnOnce(&Env, Result) -> crate::Result<()>>;
                    let then_fn = unsafe { Box::from_raw(then_fn) };
                    drop(then_fn);
                    return Ok(());
                }
            };

            tx.try_send(JsWorkerEvent::Exec {
                id: realm_id,
                callback: Box::new(move |env| {
                    let then_fn = then_fn
                        as *mut Box<dyn 'static + FnOnce(&Env, Result) -> crate::Result<()>>;
                    let then_fn = unsafe { Box::from_raw(then_fn) };
                    (*then_fn)(env, result)
                }),
            })?;

            tx.try_send(JsWorkerEvent::BackgroundTaskComplete { id: realm_id })?;

            Ok(())
        })
    }

    pub fn eval_script<Return: FromJsValue>(&self, code: impl AsRef<str>) -> crate::Result<Return> {
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

    pub fn eval_module(&self, code: impl AsRef<str>) -> crate::Result<JsObject> {
        let scope = &mut self.scope();
        let realm = JsRealm::v8_revive(scope);

        let module = Module::new(realm, generate_random_string(20), code.as_ref())?;

        let v8_module = Module::v8_run_module(true, realm, module.name.clone(), module)?;
        let v8_module = v8_module.get_module_namespace().cast::<v8::Object>();

        JsObject::from_js_value(self, Value::from(v8_module.cast()))
    }

    /// Load a file and evaluate it
    pub fn import(&self, _path: impl AsRef<Path>) -> crate::Result<()> {
        todo!()
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        // drop(unsafe { Box::from_raw(self.on_before_exit)});
        // drop(unsafe { Box::from_raw(self.shutdown_has_run) });
    }
}

// unsafe impl Send for SendFnOnce {}
// unsafe impl Sync for SendFnOnce {}

// struct SendFnOnce(*mut c_void);

// impl SendFnOnce {
//     fn new(func: Box<dyn FnOnce(&Env) -> crate::Result<()>>) -> Self {
//         let func: *mut Box<dyn Send + Sync + FnOnce(&Env) -> Result<(), crate::Error>> =
//             Box::into_raw(Box::new(func));
//         let func = func as *mut c_void;
//         Self(func as _)
//     }
// }

// impl Drop for SendFnOnce {
//     fn drop(&mut self) {
//         drop(unsafe {
//             let func: *mut (dyn FnOnce(&Env) -> crate::Result<()>) = std::mem::transmute(self.0);
//             Box::from_raw(func)
//         })
//     }
// }
