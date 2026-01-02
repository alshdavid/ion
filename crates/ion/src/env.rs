use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;
use std::sync::Arc;

use flume::Sender;
use v8::ScriptOrigin;

use crate::AsyncEnv;
use crate::FromJsValue;
use crate::JsObject;
use crate::ToJsValue;
use crate::platform::JsRealm;
use crate::platform::background_worker::BackgroundTaskManager;
use crate::platform::finalizer_registry::FinalizerRegistery;
use crate::platform::module::Module;
use crate::platform::sys;
use crate::platform::worker::JsWorkerEvent;
use crate::utils::RefCounter;
use crate::utils::hash_sha256;

#[derive(Clone)]
pub struct Env {
    pub(crate) inner: *mut Env,
    pub(crate) realm_id: usize,
    pub(crate) isolate: *mut v8::Isolate,
    pub(crate) context: sys::GlobalContext,
    pub(crate) background_task_manager: Arc<BackgroundTaskManager>,
    pub(crate) global_refs: RefCounter,
    pub(crate) shutdown_requested: Rc<RefCell<bool>>,
    pub(crate) tx: Sender<JsWorkerEvent>,
    pub(crate) finalizer_registry: FinalizerRegistery,
    pub(crate) global_this: sys::GlobalThis,
}

impl Env {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        isolate: *mut v8::Isolate,
        context: sys::GlobalContext,
        background_task_manager: Arc<BackgroundTaskManager>,
        global_refs: RefCounter,
        shutdown_requested: Rc<RefCell<bool>>,
        tx: Sender<JsWorkerEvent>,
        finalizer_registry: FinalizerRegistery,
        global_this: sys::GlobalThis,
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
            realm_id: self.realm_id,
        })
    }

    #[allow(clippy::mut_from_ref)]
    pub fn isolate(&self) -> &mut v8::Isolate {
        // SAFETY: Lifetime of `Isolate` is longer than `Env`.
        unsafe { &mut *self.isolate }
    }

    pub fn global_this(&self) -> crate::Result<JsObject> {
        let global_this = sys::Value::new(self.global_this.as_local().into());
        JsObject::from_js_value(self, global_this)
    }

    pub fn context(&self) -> v8::Local<'static, v8::Context> {
        self.context.as_local()
    }

    pub fn scope(&self) -> sys::RootScope {
        self.context.scope()
    }

    pub fn spawn_background(
        &self,
        fut: impl 'static + Send + Sync + Future<Output = crate::Result<()>>,
    ) -> crate::Result<()> {
        self.background_task_manager.spawn(fut)
    }

    pub fn eval_script_with_origin<Return: FromJsValue>(
        &self,
        code: impl AsRef<str>,
        origin: Option<&ScriptOrigin>,
    ) -> crate::Result<Return> {
        let scope = &mut self.scope();

        let Some(code) = v8::String::new(scope, code.as_ref()) else {
            panic!();
        };

        let Some(script) = v8::Script::compile(scope, code, origin) else {
            panic!();
        };

        let Some(value) = script.run(scope) else {
            panic!();
        };

        Return::from_js_value(self, sys::Value::new(value))
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

        Return::from_js_value(self, sys::Value::new(value))
    }

    pub fn eval_module(
        &self,
        code: impl AsRef<str>,
    ) -> crate::Result<JsObject> {
        // TODO cache a module based on its content hash otherwise it will leak
        let realm = JsRealm::v8_revive(self);

        let module = Module::new(realm, hash_sha256(code.as_ref().as_bytes()), code.as_ref())?;

        let v8_module = Module::v8_run_module(true, realm, module.name.clone(), module)?;
        let v8_module = v8_module.get_module_namespace().cast::<v8::Object>();

        JsObject::from_js_value(self, sys::Value::new(v8_module.into()))
    }

    /// Load a file and evaluate it
    pub fn import(
        &self,
        specifier: impl AsRef<str>,
    ) -> crate::Result<()> {
        self.tx.try_send(JsWorkerEvent::Import {
            id: self.realm_id,
            specifier: specifier.as_ref().to_string(),
        })?;

        Ok(())
    }

    /// Throw any JavaScript value.
    pub fn throw<T: ToJsValue>(
        &self,
        value: T,
    ) -> crate::Result<()> {
        let scope = &mut self.scope();
        let js_value = T::to_js_value(self, value)?;
        let v8_value = sys::v8_into_static_value::<v8::Value, v8::Value>(js_value.as_inner());
        sys::v8_throw_exception(scope, v8_value);
        Ok(())
    }

    /// Throws a JavaScript Error with the text provided.
    pub fn throw_error(
        &self,
        msg: &str,
        code: Option<&str>,
    ) -> crate::Result<()> {
        let scope = &mut self.scope();

        let message = if let Some(code) = code {
            format!("{}: {}", code, msg)
        } else {
            msg.to_string()
        };

        sys::v8_throw_error(scope, &message)
    }
}
