use crate::Env;
use crate::ToJsUnknown;
use crate::platform::sys;
use crate::platform::sys::Value;
use crate::values::FromJsValue;
use crate::values::JsObjectValue;
use crate::values::JsValue;
use crate::values::ToJsValue;

#[derive(Clone)]
pub struct JsObject {
    pub(crate) value: Value,
    pub(crate) env: Env,
}

impl JsObject {
    pub fn new(env: &Env) -> crate::Result<Self> {
        let scope = &mut env.scope();
        let object = v8::Object::new(scope);
        Ok(Self {
            value: sys::v8_from_value(object),
            env: env.clone(),
        })
    }
}

impl JsValue for JsObject {
    fn value(&self) -> &Value {
        &self.value
    }

    fn env(&self) -> &Env {
        &self.env
    }
}

impl ToJsUnknown for JsObject {}
impl JsObjectValue for JsObject {}

impl FromJsValue for JsObject {
    fn from_js_value(
        env: &Env,
        value: Value,
    ) -> crate::Result<Self> {
        Ok(Self {
            value,
            env: env.clone(),
        })
    }
}

impl ToJsValue for JsObject {
    fn to_js_value(
        _env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(val.value)
    }
}
