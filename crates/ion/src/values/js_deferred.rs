#![allow(warnings)]
use std::sync::Arc;

use flume::Sender;
use parking_lot::Mutex;

use crate::Env;
use crate::FromJsValue;
use crate::JsFunction;
use crate::JsObject;
use crate::JsObjectValue;
use crate::JsUnknown;
use crate::ThreadSafeFunction;
use crate::platform::sys;
use crate::platform::sys::Value;
use crate::thread_safe_function;
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

impl JsDeferred {
    pub fn new(env: &Env) -> crate::Result<(JsObject, JsDeferred)> {
        let global_this = env.global_this()?;
        let promise_ctor = global_this.get_named_property_unchecked::<JsFunction>("Promise")?;

        let (tx, rx) = oneshot::<JsDeferredEvent>();
        let rx = Arc::new(Mutex::new(Some(rx)));

        let receiver = JsFunction::new(env, move |env, ctx| {
            let resolve = ctx.arg::<JsFunction>(0)?;
            let reject = ctx.arg::<JsFunction>(1)?;

            let resolve = ThreadSafeFunction::new(&resolve)?;
            let reject = ThreadSafeFunction::new(&reject)?;

            env.spawn_background({
                let rx = rx.clone();
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
                            resolve
                                .call_async(
                                    move |env| {
                                        let mut lock = callback.lock();
                                        let result = lock.take().unwrap()(env)?;
                                        JsUnknown::from_js_value(env, result)
                                    },
                                    thread_safe_function::map_return::noop,
                                )
                                .await
                        }
                        Ok(JsDeferredEvent::Reject(callback)) => {
                            let callback = Mutex::new(Some(callback));
                            reject
                                .call_async(
                                    move |env| {
                                        let mut lock = callback.lock();
                                        let result = lock.take().unwrap()(env)?;
                                        JsUnknown::from_js_value(env, result)
                                    },
                                    thread_safe_function::map_return::noop,
                                )
                                .await
                        }
                        Err(_) => Err(crate::Error::PromiseResolveError),
                    }
                }
            })?;

            Ok(())
        })?;

        let promise = promise_ctor.new_instance(receiver)?;

        Ok((promise, Self { tx }))
    }

    pub fn resolve<Return: ToJsValue>(
        &self,
        callback: impl 'static + Send + Sync + FnOnce(&Env) -> crate::Result<Return>,
    ) -> crate::Result<()> {
        Ok(self
            .tx
            .try_send(JsDeferredEvent::Resolve(Box::new(move |env| {
                let value = callback(env)?;
                let value = Return::to_js_value(env, value)?;
                Ok(value)
            })))?)
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
    Resolve(Box<dyn Send + Sync + FnOnce(&Env) -> crate::Result<Value>>),
    Reject(Box<dyn Send + Sync + FnOnce(&Env) -> crate::Result<Value>>),
}
