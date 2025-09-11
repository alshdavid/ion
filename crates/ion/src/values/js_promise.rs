// TODO
use crate::Env;
use crate::ToJsUnknown;
use crate::platform::sys;
use crate::values::FromJsValue;
use crate::values::JsValue;
use crate::values::ToJsValue;

#[derive(Clone)]
pub struct JsPromise {
    pub(crate) value: sys::__v8_value,
    pub(crate) env: Env,
}

impl JsPromise {
    pub fn new(_env: &Env) -> crate::Result<JsPromise> {
        todo!()
    }
}

impl JsValue for JsPromise {
    fn value(&self) -> &sys::__v8_value {
        &self.value
    }

    fn env(&self) -> &Env {
        &self.env
    }
}

impl ToJsUnknown for JsPromise {}

impl FromJsValue for JsPromise {
    fn from_js_value(
        env: &Env,
        value: sys::__v8_value,
    ) -> crate::Result<Self> {
        Ok(Self {
            value,
            env: env.clone(),
        })
    }
}

impl ToJsValue for JsPromise {
    fn to_js_value(
        _env: &Env,
        val: Self,
    ) -> crate::Result<sys::__v8_value> {
        Ok(val.value)
    }
}
