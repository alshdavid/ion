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

        let handle = value.inner();
        let inner = v8::Global::new(scope, handle);

        let (tx, rx) = unbounded::<ThreadSafeFunctionEvent>();

        env.on_before_exit({
            let tx = tx.clone();
            move || Ok(tx.try_send(ThreadSafeFunctionEvent::Shutdown)?)
        });

        env.spawn_local({
            let env = *env;
            async move {
                let ref_count = RefCounter::new(1);
                let mut can_shutdown = env.shutdown_has_run();
                let inner = inner;

                while let Ok(event) = rx.recv_async().await {
                    match event {
                        ThreadSafeFunctionEvent::Call {
                            map_arguments,
                            map_return,
                        } => {
                            let scope = &mut env.scope();
                            let func = v8::Local::new(scope, inner.clone());
                            let func = func.cast::<v8::Function>();
                            let recv = v8::undefined(scope);
                            let arguments = map_arguments(&env)?;
                            let ret = match func.call(scope, recv.into(), &arguments) {
                                Some(value) => value,
                                None => v8::undefined(scope).into(),
                            };
                            let ret = JsUnknown::from_js_value(&env, Value::from(ret))?;
                            map_return(&env, ret)?;
                        }
                        ThreadSafeFunctionEvent::Ref => {
                            ref_count.inc();
                        }
                        ThreadSafeFunctionEvent::Unref => {
                            if !ref_count.dec() {
                                continue;
                            }
                            if can_shutdown {
                                break;
                            }
                        }
                        ThreadSafeFunctionEvent::Shutdown => {
                            can_shutdown = true;
                            if ref_count.count() == 0 {
                                break;
                            }
                        }
                    }
                }
                Ok(())
            }
        })?;

        Ok(Self { tx })
    }

    pub fn call<Args: JsValuesTupleIntoVec>(
        &self,
        map_arguments: impl 'static + Fn(&Env) -> crate::Result<Args>,
        map_return: impl 'static + Fn(&Env, JsUnknown) -> crate::Result<()>,
    ) -> crate::Result<()> {
        self.tx.try_send(ThreadSafeFunctionEvent::Call {
            map_arguments: Box::new(move |env| {
                Ok(map_arguments(env)?
                    .into_vec(env)?
                    .iter()
                    .map(|v| v.inner())
                    .collect::<Vec<_>>())
            }),
            map_return: Box::new(move |env, ret| map_return(env, ret)),
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
    Call {
        map_arguments: Box<dyn Fn(&Env) -> crate::Result<Vec<v8::Local<'static, v8::Value>>>>,
        map_return: Box<dyn Fn(&Env, JsUnknown) -> crate::Result<()>>,
    },
    Ref,
    Unref,
    Shutdown,
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
