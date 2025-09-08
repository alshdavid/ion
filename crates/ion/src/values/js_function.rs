use std::ffi::c_int;

use crate::Env;
use crate::JsObject;
use crate::JsObjectValue;
use crate::JsValuesTupleIntoVec;
use crate::ToJsUnknown;
use crate::platform::Reference;
use crate::platform::ReferenceOwnership;
use crate::platform::Value;
use crate::utils::v8::v8_create_undefined;
use crate::values::FromJsValue;
use crate::values::JsValue;
use crate::values::ToJsValue;

#[derive(Debug, Clone)]
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

        Reference::register_global_finalizer(
            external,
            env.into_raw(),
            1,
            ReferenceOwnership::Rust,
            Some(Box::new(move |_| drop(unsafe { Box::from_raw(callback) }))),
        );

        let value = v8::Function::builder(
            |_scope: &mut v8::HandleScope,
             args: v8::FunctionCallbackArguments,
             mut rv: v8::ReturnValue| {
                let js_data = args.data();
                let js_external = js_data.try_cast::<v8::External>().unwrap();
                let external_ptr =
                    js_external.value() as *mut JsFunctionCallbackInfo<Return, Callback>;
                let info: &JsFunctionCallbackInfo<Return, Callback> = unsafe { &*external_ptr };

                let env = unsafe { Env::from_raw(info.env) };
                let ctx = JsFunctionCallContext {
                    env: info.env,
                    args,
                };
                let callback = &info.callback;
                let result = callback(&env, ctx).unwrap();
                let result = Return::to_js_value(&env, result).unwrap();
                rv.set(result.inner());
            },
        )
        .data(external.into())
        .build(scope)
        .unwrap();

        Ok(Self {
            value: Value::from(value.cast()),
            this: None,
            env: *env,
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
            args_v8.push(arg.inner());
        }

        let local = self.value.inner();
        let local = local.cast::<v8::Function>();

        let this = match &self.this {
            Some(this) => this.inner(),
            None => v8_create_undefined(scope)?,
        };

        let result = match local.call(scope, this, &args_v8) {
            Some(result) => result,
            None => v8_create_undefined(scope)?,
        };

        let value = Value::from(result);
        Return::from_js_value(&self.env, value)
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
            args_v8.push(arg.inner());
        }

        let local = self.value.inner();
        let local = local.cast::<v8::Function>();

        let Some(result) = local.new_instance(scope, &args_v8) else {
            return Err(crate::Error::NewInstanceError);
        };

        let value = Value::from(result.cast());
        JsObject::from_js_value(&self.env, value)
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
            env: *env,
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
        let value = Value::from(self.args.get(i));
        let value = Arg::from_js_value(&unsafe { Env::from_raw(self.env) }, value)?;
        Ok(value)
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
