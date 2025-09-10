#![allow(warnings)]
use flume::Sender;
use flume::unbounded;

use crate::Env;
use crate::FromJsValue;
use crate::JsFunction;
use crate::JsUnknown;
use crate::JsValue;
use crate::JsValuesTupleIntoVec;
use crate::platform::Value;
use crate::utils::RefCounter;
use crate::utils::channel::oneshot;

pub struct ThreadSafeFunction {
    tx: Sender<ThreadSafeFunctionEvent>,
}

impl ThreadSafeFunction {
    pub fn new(target: &JsFunction) -> crate::Result<Self> {
        let value = target.value();
        let env = target.env();
        let scope = &mut env.scope();

        // Create threadsafe function with an initial refcount of 1
        env.inc_ref();

        // SAFETY: Force function to be Send + Sync
        let handle = value.inner();
        let inner = v8::Global::new(scope, handle);
        let inner = Box::new(inner);
        let inner = Box::into_raw(Box::new(inner));
        let inner = inner as usize;

        let (tx, rx) = unbounded::<ThreadSafeFunctionEvent>();

        env.spawn_background(move |env| {
            Box::pin(async move {
                let inner = inner;

                while let Ok(event) = rx.recv_async().await {
                    match event {
                        ThreadSafeFunctionEvent::Call {
                            map_arguments,
                            map_return,
                        } => {
                            env.exec(move |env| {
                                let scope = &mut env.scope();

                                let inner = inner as *mut Box<v8::Local<'static, v8::Function>>;
                                let inner = unsafe { &*inner };

                                let map_arguments = map_arguments
                                    as *mut Box<
                                        dyn Fn(
                                            &Env,
                                        )
                                            -> crate::Result<Vec<v8::Local<'static, v8::Value>>>,
                                    >;
                                let map_arguments = unsafe { *Box::from_raw(map_arguments) };
                                let arguments = map_arguments(&env)?;

                                let map_return = map_return
                                    as *mut Box<dyn Fn(&Env, JsUnknown) -> crate::Result<()>>;
                                let map_return = unsafe { *Box::from_raw(map_return) };

                                let recv = v8::undefined(scope);
                                let ret = inner.call(scope, recv.into(), &arguments).unwrap();

                                let ret = JsUnknown::from_js_value(&env, Value::from(ret))?;
                                map_return(&env, ret)?;

                                Ok(())
                            })?;
                        }
                        ThreadSafeFunctionEvent::Ref => {
                            env.exec_async(move |env| {
                                env.inc_ref();
                                Ok(())
                            }).await?;
                        }
                        ThreadSafeFunctionEvent::Unref => {
                            env.exec_async(move |env| {
                                env.dec_ref();
                                Ok(())
                            }).await?;
                        }
                    };
                }

                // Clean up
                env.exec_async(move |env| {
                    let inner = inner as *mut Box<v8::Global<v8::Function>>;
                    let inner = unsafe { Box::from_raw(inner) };

                    Ok(())
                })
                .await?;

                Ok(())
            })
        })?;

        Ok(Self { tx })
    }

    pub fn call<Args: JsValuesTupleIntoVec>(
        &self,
        map_arguments: impl 'static + Fn(&Env) -> crate::Result<Args>,
        map_return: impl 'static + Fn(&Env, JsUnknown) -> crate::Result<()>,
    ) -> crate::Result<()> {
        let map_arguments: Box<
            Box<dyn Fn(&Env) -> crate::Result<Vec<v8::Local<'static, v8::Value>>>>,
        > = Box::new(Box::new(
            move |env| -> crate::Result<Vec<v8::Local<'static, v8::Value>>> {
                let mut result = vec![];
                for value in map_arguments(env)?.into_vec(env)? {
                    result.push(value.into_inner());
                }
                Ok(result)
            },
        ));
        let map_arguments = Box::into_raw(map_arguments);
        let map_arguments = map_arguments as usize;

        let map_return: Box<Box<dyn Fn(&Env, JsUnknown) -> crate::Result<()>>> =
            Box::new(Box::new(move |env, ret| -> crate::Result<()> {
                map_return(env, ret)
            }));
        let map_return = Box::into_raw(map_return);
        let map_return = map_return as usize;

        self.tx.try_send(ThreadSafeFunctionEvent::Call {
            map_arguments,
            map_return,
        })?;
        Ok(())
    }

    pub fn call_blocking<Args: JsValuesTupleIntoVec, Return: 'static>(
        &self,
        map_arguments: impl 'static + Fn(&Env) -> crate::Result<Args>,
        map_return: impl 'static + Fn(&Env, JsUnknown) -> crate::Result<Return>,
    ) -> crate::Result<Return> {
        let (tx, rx) = oneshot();
        self.call(map_arguments, move |env, ret| {
            Ok(tx.try_send(map_return(env, ret))?)
        })?;
        rx.recv()?
    }

    pub async fn call_async<Args: JsValuesTupleIntoVec, Return: 'static>(
        &self,
        map_arguments: impl 'static + Fn(&Env) -> crate::Result<Args>,
        map_return: impl 'static + Fn(&Env, JsUnknown) -> crate::Result<Return>,
    ) -> crate::Result<Return> {
        let (tx, rx) = oneshot();
        self.call(map_arguments, move |env, ret| {
            Ok(tx.try_send(map_return(env, ret))?)
        })?;
        rx.recv_async().await?
    }

    pub fn inc_ref(&self) -> crate::Result<()> {
        Ok(self.tx.try_send(ThreadSafeFunctionEvent::Ref)?)
    }

    pub fn dec_ref(&self) -> crate::Result<()> {
        Ok(self.tx.try_send(ThreadSafeFunctionEvent::Unref)?)
    }
}

unsafe impl Send for ThreadSafeFunction {}
unsafe impl Sync for ThreadSafeFunction {}

impl Clone for ThreadSafeFunction {
    fn clone(&self) -> Self {
        drop(self.tx.try_send(ThreadSafeFunctionEvent::Ref));
        Self {
            tx: self.tx.clone(),
        }
    }
}

impl Drop for ThreadSafeFunction {
    fn drop(&mut self) {
        drop(self.tx.try_send(ThreadSafeFunctionEvent::Unref));
    }
}

#[allow(clippy::type_complexity)]
enum ThreadSafeFunctionEvent {
    /// Force callbacks to be Send + Sync
    Call {
        /// Box<dyn Fn(&Env) -> crate::Result<Vec<v8::Local<'static, v8::Value>>>>
        map_arguments: usize,
        /// Box<dyn Fn(&Env, JsUnknown) -> crate::Result<()>>
        map_return: usize,
    },
    Ref,
    Unref,
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

    pub fn noop(_env: &Env, _ret: JsUnknown) -> crate::Result<()> {
        Ok(())
    }
}
