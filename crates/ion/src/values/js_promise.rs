// TODO
use crate::Env;
use crate::ToJsUnknown;
use crate::platform::Value;
use crate::values::FromJsValue;
use crate::values::JsValue;
use crate::values::ToJsValue;

#[derive(Clone)]
pub struct JsPromise {
    pub(crate) value: Value,
    pub(crate) env: Env,
}

impl JsPromise {
    pub fn new(_env: &Env) -> crate::Result<JsPromise> {
        todo!()
    }
}

impl JsValue for JsPromise {
    fn value(&self) -> &Value {
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
        value: Value,
    ) -> crate::Result<Self> {
        Ok(Self { value, env: *env })
    }
}

impl ToJsValue for JsPromise {
    fn to_js_value(
        _env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(val.value)
    }
}
