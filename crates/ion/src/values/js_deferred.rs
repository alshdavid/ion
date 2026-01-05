use std::sync::Arc;

use flume::Sender;
use parking_lot::Mutex;

use crate::Env;
use crate::FromJsValue;
use crate::JsObject;
use crate::platform::sys;
use crate::utils::channel::oneshot;
use crate::values::ToJsValue;

/// JsDeferred is a type that allows for waiting on asynchronous
/// behavior, returning a Promise that can be externally resolved.
///
/// You can think of this as essentially a oneshot channel that
/// returns a Promise to JavaScript
pub struct JsDeferred {
    tx: Sender<JsDeferredEvent>,
}

struct SendableResolver(v8::Global<v8::PromiseResolver>);
unsafe impl Send for SendableResolver {}
unsafe impl Sync for SendableResolver {}

impl SendableResolver {
    fn into_local<'a>(
        self,
        scope: &mut v8::PinnedRef<'a, v8::HandleScope<'a, v8::Context>>,
    ) -> v8::Local<'static, v8::PromiseResolver> {
        
        unsafe {
            std::mem::transmute::<v8::Local<'a, v8::PromiseResolver>, v8::Local<'static, v8::PromiseResolver>>(v8::Local::new(scope, self.0))
        }
    }
}

impl JsDeferred {
    pub fn new(env: &Env) -> crate::Result<(JsObject, JsDeferred)> {
        let scope = &mut env.scope();
        let isolate = env.isolate();

        env.inc_ref();

        let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
        let promise = promise_resolver.get_promise(scope);
        let promise_resolver_global = SendableResolver(v8::Global::new(isolate, promise_resolver));


        let (tx, rx) = oneshot::<JsDeferredEvent>();
        let rx = Arc::new(Mutex::new(Some(rx)));

        env.spawn_background({
            let rx = rx.clone();
            let env = env.as_async();

            async move {
                let rx = {
                    let mut lock = rx.lock();
                    let Some(rx) = lock.take() else {
                        return Err(crate::Error::PromiseResolveError);
                    };
                    rx
                };

                match rx.recv_async().await {
                    Ok(JsDeferredEvent::Resolve(callback)) => {
                        let callback = Mutex::new(Some(callback));

                        env.exec_async(move |env| {
                            let scope = &mut env.scope();

                            let promise_resolver_local = promise_resolver_global.into_local(scope);

                            let mut lock = callback.lock();
                            let result = lock.take().unwrap()(env)?;

                            promise_resolver_local.resolve(scope, result.as_inner());
                            env.dec_ref();
                            Ok(())
                        }).await?;
                    }
                    Ok(JsDeferredEvent::Reject(callback)) => {
                        println!("3.3.1");
                        todo!()
                    }
                    Err(err) => {
                        println!("3.4.1 {:?}", err);
                        panic!()
                    },
                }


                Ok(())

            }
        })?;

        Ok((
            JsObject::from_js_value(env, sys::Value::new(promise.into()))?,
            Self { tx },
        ))
    }

    pub fn resolve<Return: ToJsValue>(
        &self,
        callback: impl 'static + Send + Sync + FnOnce(&Env) -> crate::Result<Return>,
    ) -> crate::Result<()> {
        let (tx, rx) = oneshot::<()>();

        self
            .tx
            .try_send(JsDeferredEvent::Resolve(Box::new(move |env| {
                let value = callback(env)?;
                let value = Return::to_js_value(env, value)?;
                tx.send(()).unwrap();
                Ok(value)
            })))?;

        rx.recv().unwrap(); 

        Ok(())
    }

    pub fn reject<Return: ToJsValue>(
        &self,
        callback: impl 'static + Send + Sync + FnOnce(&Env) -> crate::Result<Return>,
    ) -> crate::Result<()> {
        Ok(self
            .tx
            .try_send(JsDeferredEvent::Reject(Box::new(move |env| {
                let value = callback(env)?;
                let value = Return::to_js_value(env, value)?;
                Ok(value)
            })))?)
    }
}

unsafe impl Send for JsDeferred {}
unsafe impl Sync for JsDeferred {}

#[allow(clippy::type_complexity)]
enum JsDeferredEvent {
    Resolve(Box<dyn Send + Sync + FnOnce(&Env) -> crate::Result<sys::Value>>),
    Reject(Box<dyn Send + Sync + FnOnce(&Env) -> crate::Result<sys::Value>>),
}
