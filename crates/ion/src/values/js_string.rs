use std::rc::Rc;
use std::sync::Arc;

use crate::Env;
use crate::ToJsUnknown;
use crate::platform::Value;
use crate::values::FromJsValue;
use crate::values::JsObjectValue;
use crate::values::JsValue;
use crate::values::ToJsValue;

#[derive(Clone)]
pub struct JsString {
    pub(crate) value: Value,
    pub(crate) env: Env,
}

impl JsString {
    pub fn new(
        env: &Env,
        text: impl AsRef<str>,
    ) -> crate::Result<Self> {
        let scope = &mut env.scope();
        let string = crate::utils::v8::v8_create_string(scope, text.as_ref())?;
        Ok(Self {
            value: Value::from(string.cast()),
            env: *env,
        })
    }

    pub fn get_string(&self) -> crate::Result<String> {
        let scope = &mut self.env.scope();
        let local = self.value.inner();
        let local = local.cast::<v8::String>();
        Ok(local.to_rust_string_lossy(scope))
    }
}

impl JsValue for JsString {
    fn value(&self) -> &Value {
        &self.value
    }

    fn env(&self) -> &Env {
        &self.env
    }
}

impl ToJsUnknown for JsString {}
impl JsObjectValue for JsString {}

impl FromJsValue for JsString {
    fn from_js_value(
        env: &Env,
        value: Value,
    ) -> crate::Result<Self> {
        Ok(Self { value, env: *env })
    }
}

impl ToJsValue for JsString {
    fn to_js_value(
        _env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(val.value)
    }
}

impl ToJsValue for String {
    fn to_js_value(
        env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(*JsString::new(env, val)?.value())
    }
}

impl ToJsValue for &str {
    fn to_js_value(
        env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(*JsString::new(env, val)?.value())
    }
}

impl ToJsValue for Rc<str> {
    fn to_js_value(
        env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(*JsString::new(env, val)?.value())
    }
}

impl ToJsValue for Arc<str> {
    fn to_js_value(
        env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(*JsString::new(env, val)?.value())
    }
}

impl Env {
    pub fn create_string(
        &self,
        value: impl AsRef<str>,
    ) -> crate::Result<JsString> {
        JsString::new(self, value)
    }
}
