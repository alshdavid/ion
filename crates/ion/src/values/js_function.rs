use std::ffi::c_int;

use crate::Env;
use crate::JsObject;
use crate::JsObjectValue;
use crate::JsUndefined;
use crate::JsUnknown;
use crate::JsValuesTupleIntoVec;
use crate::ToJsUnknown;
use crate::platform::sys;
use crate::platform::sys::Value;
use crate::values::FromJsValue;
use crate::values::JsValue;
use crate::values::ToJsValue;

#[derive(Clone)]
pub struct JsFunction {
    pub(crate) value: Value,
    pub(crate) this: Option<Value>,
    pub(crate) env: Env,
}

impl JsFunction {
    pub fn new<Return, Callback>(
        env: &Env,
        callback: Callback,
    ) -> crate::Result<Self>
    where
        Return: 'static + ToJsValue,
        Callback: 'static + Fn(&Env, JsFunctionCallContext) -> crate::Result<Return>,
    {
        let scope = &mut env.scope();

        let callback = Box::into_raw(Box::new(JsFunctionCallbackInfo {
            env: env.into_raw(),
            callback,
        }));

        let external = v8::External::new(scope, callback as _);
        env.finalizer_registry.register(&external.into(), {
            move || {
                drop(unsafe { Box::from_raw(callback) });
            }
        });

        let value = v8::Function::builder(
            |_scope: &mut v8::PinnedRef<'_, v8::HandleScope<'_, v8::Context>>,
             args: v8::FunctionCallbackArguments,
             mut rv: v8::ReturnValue| {
                let js_data = args.data();
                let js_external = js_data.try_cast::<v8::External>().unwrap();
                let external_ptr =
                    js_external.value() as *mut JsFunctionCallbackInfo<Return, Callback>;
                let info: &JsFunctionCallbackInfo<Return, Callback> = unsafe { &*external_ptr };

                let env = unsafe { Env::from_raw(info.env) };
                let this = args.this();
                let this_js_unknown =
                    match JsUnknown::from_js_value(&env, sys::Value::new(this.into())) {
                        Ok(value) => value,
                        Err(_) => JsUndefined::new(&env).unwrap().into_unknown(),
                    };

                let ctx = JsFunctionCallContext {
                    env: info.env,
                    args,
                    this: this_js_unknown,
                };
                let callback = &info.callback;
                let result = callback(&env, ctx).unwrap();
                let result = Return::to_js_value(&env, result).unwrap();
                rv.set(result.as_inner());
            },
        )
        .data(external.into())
        .build(scope)
        .expect("Failed to create function");

        Ok(Self {
            value: sys::Value::new(value.into()),
            this: None,
            env: env.clone(),
        })
    }

    pub fn call<Return>(&self) -> crate::Result<Return>
    where
        Return: FromJsValue,
    {
        self.call_with_args::<Return, ()>(())
    }

    pub fn call_with_args<Return, Args>(
        &self,
        args: Args,
    ) -> crate::Result<Return>
    where
        Args: JsValuesTupleIntoVec,
        Return: FromJsValue,
    {
        let scope = &mut self.env.scope();

        let args = args.into_vec(&self.env)?;
        let mut args_v8 = vec![];
        for arg in args {
            args_v8.push(arg.as_inner());
        }

        let local = self.value.cast::<v8::Function>();

        let this = match &self.this {
            Some(this) => this.as_inner(),
            None => JsUndefined::new(&self.env)?.value.as_inner(),
        };

        let result = match local.call(scope, this, &args_v8) {
            Some(result) => result,
            None => JsUndefined::new(&self.env)?.value.as_inner(),
        };

        Return::from_js_value(&self.env, sys::Value::new(result))
    }

    pub fn call_with_args_with_recv<Return, Args>(
        &self,
        args: Args,
        recv: &impl JsValue,
    ) -> crate::Result<Return>
    where
        Args: JsValuesTupleIntoVec,
        Return: FromJsValue,
    {
        let scope = &mut self.env.scope();

        let args = args.into_vec(&self.env)?;
        let mut args_v8 = vec![];
        for arg in args {
            args_v8.push(arg.as_inner());
        }

        let local = self.value.cast::<v8::Function>();

        let this = recv.value().as_inner();

        let result = match local.call(scope, this, &args_v8) {
            Some(result) => result,
            None => JsUndefined::new(&self.env)?.value.as_inner(),
        };

        Return::from_js_value(&self.env, sys::Value::new(result))
    }

    pub fn new_instance<Args>(
        &self,
        args: Args,
    ) -> crate::Result<JsObject>
    where
        Args: JsValuesTupleIntoVec,
    {
        let scope = &mut self.env.scope();

        let args = args.into_vec(&self.env)?;
        let mut args_v8 = vec![];
        for arg in args {
            args_v8.push(arg.as_inner());
        }

        let local = self.value.cast::<v8::Function>();

        let Some(result) = local.new_instance(scope, &args_v8) else {
            return Err(crate::Error::NewInstanceError);
        };

        JsObject::from_js_value(&self.env, sys::Value::new(result.into()))
    }
}

impl JsValue for JsFunction {
    fn value(&self) -> &Value {
        &self.value
    }

    fn env(&self) -> &Env {
        &self.env
    }
}

impl ToJsUnknown for JsFunction {}
impl JsObjectValue for JsFunction {}

impl FromJsValue for JsFunction {
    fn from_js_value(
        env: &Env,
        value: Value,
    ) -> crate::Result<Self> {
        Ok(Self {
            value,
            this: None,
            env: env.clone(),
        })
    }
}

impl ToJsValue for JsFunction {
    fn to_js_value(
        _env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(val.value)
    }
}

pub struct JsFunctionCallContext<'a> {
    env: *mut Env,
    args: v8::FunctionCallbackArguments<'a>,
    this: JsUnknown,
}

impl<'a> JsFunctionCallContext<'a> {
    pub fn arg<Arg: FromJsValue>(
        &self,
        i: i32,
    ) -> crate::Result<Arg> {
        let len = self.args.length();
        if i >= len {
            return Err(crate::Error::OutOfBounds);
        }
        let Ok(i) = c_int::try_from(i);
        let v8_arg = self.args.get(i);
        let v8_value = sys::v8_into_static_value(v8_arg);
        let value = Arg::from_js_value(
            &unsafe { Env::from_raw(self.env) },
            sys::Value::new(v8_value),
        )?;
        Ok(value)
    }

    pub fn args(&self) -> crate::Result<Vec<JsUnknown>> {
        let env = unsafe { Env::from_raw(self.env) };
        let mut result = Vec::<JsUnknown>::new();

        for i in 0..self.args.length() {
            let v8_arg = self.args.get(i);
            let v8_value = sys::v8_into_static_value(v8_arg);
            let value = JsUnknown::from_js_value(&env, sys::Value::new(v8_value))?;
            result.push(value);
        }

        Ok(result)
    }

    pub fn this(&self) -> JsUnknown {
        self.this.clone()
    }

    pub fn len(&self) -> i32 {
        self.args.length()
    }

    pub fn is_empty(&self) -> bool {
        self.args.length() == 0
    }
}

struct JsFunctionCallbackInfo<R, T: 'static + Fn(&Env, JsFunctionCallContext) -> crate::Result<R>> {
    pub env: *mut Env,
    pub callback: T,
}
