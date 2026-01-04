use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use crate::AsyncEnv;
use crate::Env;
use crate::FromJsValue;
use crate::JsPromise;
use crate::JsPromiseResult;
use crate::JsValue;
use crate::utils::channel::oneshot;

pub struct ThreadSafePromise {
    ref_count: Arc<AtomicUsize>,
    env: Arc<AsyncEnv>,
    /// Box<v8::Global<v8::Object>>
    inner: usize,
}

impl ThreadSafePromise {
    pub fn new(target: &JsPromise) -> crate::Result<Self> {
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

    /// Non blocking call to then/catch
    pub fn settled<Resolved: FromJsValue + 'static>(
        self,
        settled_callback: impl 'static
        + Send
        + Sync
        + FnOnce(&Env, JsPromiseResult<Resolved>) -> crate::Result<()>,
    ) -> crate::Result<()> {
        let inner = self.inner;

        self.env.exec(move |env| {
            let scope = &mut env.scope();

            let inner = inner as *const v8::Local<'static, v8::Value>;
            let inner = unsafe { *inner };
            let inner = v8::Local::new(scope, inner);

            let promise = JsPromise::from_js_value(env, inner)?;

            promise.settled(settled_callback)
        })
    }

    /// Blocking call to then/catch
    pub fn settled_blocking<Resolved: FromJsValue + 'static, Return: 'static + Send + Sync>(
        self,
        settled_callback: impl 'static
        + Send
        + Sync
        + FnOnce(&Env, JsPromiseResult<Resolved>) -> crate::Result<Return>,
    ) -> crate::Result<Return> {
        let (tx, rx) = oneshot();
        self.settled(move |env, result| {
            tx.try_send(settled_callback(env, result)?)?;
            Ok(())
        })?;
        Ok(rx.recv()?)
    }

    /// Async blocking call to then/catch
    pub async fn settled_async<Resolved: FromJsValue + 'static, Return: 'static + Send + Sync>(
        self,
        settled_callback: impl 'static
        + Send
        + Sync
        + FnOnce(&Env, JsPromiseResult<Resolved>) -> crate::Result<Return>,
    ) -> crate::Result<Return> {
        let (tx, rx) = oneshot();
        self.settled(move |env, result| {
            tx.try_send(settled_callback(env, result)?)?;
            Ok(())
        })?;
        Ok(rx.recv_async().await?)
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
                let inner = inner as *mut v8::Global<v8::Object>;
                drop(unsafe { Box::from_raw(inner) });
                env.dec_ref();
                Ok(())
            })?;
        }
        Ok(())
    }
}

unsafe impl Send for ThreadSafePromise {}
unsafe impl Sync for ThreadSafePromise {}

impl Clone for ThreadSafePromise {
    fn clone(&self) -> Self {
        drop(self.inc_ref());
        Self {
            ref_count: Arc::clone(&self.ref_count),
            env: Arc::clone(&self.env),
            inner: self.inner,
        }
    }
}

impl Drop for ThreadSafePromise {
    fn drop(&mut self) {
        drop(self.dec_ref());
    }
}
