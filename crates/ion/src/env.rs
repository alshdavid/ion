use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

use flume::Sender;

use crate::AsyncEnv;
use crate::FromJsValue;
use crate::JsObject;
use crate::platform::JsRealm;
use crate::platform::background_worker::BackgroundTaskManager;
use crate::platform::finalizer_registry::FinalizerRegistery;
use crate::platform::module::Module;
use crate::platform::sys;
use crate::platform::worker::JsWorkerEvent;
use crate::utils::RefCounter;
use crate::utils::generate_random_string;

#[derive(Clone)]
pub struct Env {
    pub(crate) inner: *mut Env,
    pub(crate) realm_id: usize,
    pub(crate) isolate: *mut v8::Isolate,
    pub(crate) context: sys::__v8_context,
    pub(crate) global_this: sys::__v8_global_this,
    pub(crate) background_task_manager: Arc<BackgroundTaskManager>,
    pub(crate) global_refs: RefCounter,
    pub(crate) shutdown_requested: Rc<RefCell<bool>>,
    pub(crate) tx: Sender<JsWorkerEvent>,
    pub(crate) finalizer_registry: FinalizerRegistery,
}

impl Env {
    pub(crate) fn new(
        isolate: *mut v8::Isolate,
        context: sys::__v8_context,
        global_this: sys::__v8_global_this,
        background_task_manager: Arc<BackgroundTaskManager>,
        global_refs: RefCounter,
        shutdown_requested: Rc<RefCell<bool>>,
        tx: Sender<JsWorkerEvent>,
        finalizer_registry: FinalizerRegistery,
    ) -> Box<Self> {
        let mut env = Box::new(Env {
            realm_id: 0,
            isolate,
            context,
            global_this,
            background_task_manager,
            inner: std::ptr::null_mut(),
            global_refs,
            shutdown_requested,
            finalizer_registry,
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
        let shutdown_requested = {
            let shutdown_requested = self.shutdown_requested.borrow();
            *shutdown_requested
        };

        if self.global_refs.count() == 0 && shutdown_requested {
            self.tx
                .try_send(JsWorkerEvent::RequestContextShutdown {
                    id: self.realm_id,
                    resolve: None,
                })
                .unwrap();
        }
    }

    pub fn ref_count(&self) -> usize {
        self.global_refs.count()
    }

    pub fn as_async(&self) -> Arc<AsyncEnv> {
        Arc::new(AsyncEnv {
            tx: self.tx.clone(),
            realm_id: self.realm_id.clone(),
        })
    }

    pub fn isolate(&mut self) -> &mut v8::Isolate {
        // SAFETY: Lifetime of `Isolate` is longer than `Env`.
        unsafe { &mut *self.isolate }
    }

    pub fn global_this(&self) -> crate::Result<JsObject> {
        let v = sys::v8_get_global_this(self.global_this);
        JsObject::from_js_value(self, sys::v8_from_value(v))
    }

    pub fn context(&self) -> v8::Local<'static, v8::Context> {
        sys::v8_get_context(self.context)
    }

    pub fn scope(&self) -> v8::CallbackScope<'static> {
        let context = sys::v8_get_context(self.context);
        unsafe { v8::CallbackScope::new(context) }
    }

    pub fn spawn_background(
        &self,
        fut: impl 'static + Send + Sync + Future<Output = crate::Result<()>>,
    ) -> crate::Result<()> {
        self.background_task_manager.spawn(fut)
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

        Return::from_js_value(self, sys::v8_from_value(value))
    }

    pub fn eval_module(
        &self,
        code: impl AsRef<str>,
    ) -> crate::Result<JsObject> {
        // TODO cache a module based on its content hash otherwise it will leak
        let scope = &mut self.scope();
        let realm = JsRealm::v8_revive(scope);

        let module = Module::new(realm, generate_random_string(20), code.as_ref())?;

        let v8_module = Module::v8_run_module(true, realm, module.name.clone(), module)?;
        let v8_module = v8_module.get_module_namespace().cast::<v8::Object>();

        JsObject::from_js_value(self, sys::v8_from_value(v8_module))
    }

    /// Load a file and evaluate it
    pub fn import(
        &self,
        _path: impl AsRef<Path>,
    ) -> crate::Result<()> {
        todo!()
    }
}
