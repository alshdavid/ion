use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use crate::AsyncEnv;
use crate::Env;
use crate::FromJsValue;
use crate::JsFunction;
use crate::JsUnknown;
use crate::JsValue;
use crate::JsValuesTupleIntoVec;
use crate::utils::channel::oneshot;

pub struct ThreadSafeFunction {
    ref_count: Arc<AtomicUsize>,
    env: Arc<AsyncEnv>,
    /// Box<v8::Global<v8::Value>>
    inner: usize,
}

impl ThreadSafeFunction {
    pub fn new(target: &JsFunction) -> crate::Result<Self> {
        let env = target.env();
        let scope = &mut env.scope();

        // Create threadsafe function with an initial refcount of 1
        let ref_count = Arc::new(AtomicUsize::new(1));
        // Indicate that the current environment cannot exit until the ref_count is 0
        env.inc_ref();

        // SAFETY: Force function to be Send + Sync
        let inner = *target.value();
        let inner = v8::Global::new(scope, inner);
        let inner = Box::new(inner);
        let inner = Box::into_raw(inner);
        let inner = inner as usize;

        Ok(Self {
            ref_count,
            env: env.as_async(),
            inner,
        })
    }

    pub fn call<Args: JsValuesTupleIntoVec>(
        &self,
        map_arguments: impl 'static + Send + Sync + FnOnce(&Env) -> crate::Result<Args>,
        map_return: impl 'static + Send + Sync + FnOnce(&Env, JsUnknown) -> crate::Result<()>,
    ) -> crate::Result<()> {
        let inner = self.inner;

        self.env.exec(move |env| {
            let scope = &mut env.scope();

            let inner = inner as *const v8::Local<'static, v8::Function>;
            let inner = unsafe { *inner };
            let inner = v8::Local::new(scope, inner);

            let arguments = map_arguments(env)?.into_vec(env)?;

            let recv = v8::undefined(scope);
            let ret = inner.call(scope, recv.into(), &arguments).unwrap();

            let ret = JsUnknown::from_js_value(env, ret)?;
            map_return(env, ret)?;

            Ok(())
        })
    }

    pub fn call_blocking<Args: JsValuesTupleIntoVec, Return: 'static + Send + Sync>(
        &self,
        map_arguments: impl 'static + Send + Sync + FnOnce(&Env) -> crate::Result<Args>,
        map_return: impl 'static + Send + Sync + FnOnce(&Env, JsUnknown) -> crate::Result<Return>,
    ) -> crate::Result<Return> {
        let (tx, rx) = oneshot();
        self.call(map_arguments, move |env, ret| {
            Ok(tx.try_send(map_return(env, ret))?)
        })?;
        rx.recv()?
    }

    pub async fn call_async<Args: JsValuesTupleIntoVec, Return: 'static + Send + Sync>(
        &self,
        map_arguments: impl 'static + Send + Sync + FnOnce(&Env) -> crate::Result<Args>,
        map_return: impl 'static + Send + Sync + FnOnce(&Env, JsUnknown) -> crate::Result<Return>,
    ) -> crate::Result<Return> {
        let (tx, rx) = oneshot();
        self.call(map_arguments, move |env, ret| {
            Ok(tx.try_send(map_return(env, ret))?)
        })?;
        rx.recv_async().await?
    }

    pub fn inc_ref(&self) -> crate::Result<()> {
        self.ref_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    pub fn dec_ref(&self) -> crate::Result<()> {
        let previous = self.ref_count.fetch_sub(1, Ordering::Relaxed);
        if previous == 1 {
            let inner = self.inner;
            self.env.exec(move |env| {
                let inner = inner as *mut v8::Global<v8::Function>;
                drop(unsafe { Box::from_raw(inner) });
                env.dec_ref();
                Ok(())
            })?;
        }
        Ok(())
    }
}

unsafe impl Send for ThreadSafeFunction {}
unsafe impl Sync for ThreadSafeFunction {}

impl Clone for ThreadSafeFunction {
    fn clone(&self) -> Self {
        drop(self.inc_ref());
        Self {
            ref_count: Arc::clone(&self.ref_count),
            env: Arc::clone(&self.env),
            inner: self.inner,
        }
    }
}

impl Drop for ThreadSafeFunction {
    fn drop(&mut self) {
        drop(self.dec_ref());
    }
}

pub mod map_arguments {
    use crate::Env;

    pub fn noop(_env: &Env) -> crate::Result<()> {
        Ok(())
    }
}

pub mod map_return {
    use crate::Env;
    use crate::JsUnknown;

    pub fn noop(
        _env: &Env,
        _ret: JsUnknown,
    ) -> crate::Result<()> {
        Ok(())
    }
}
