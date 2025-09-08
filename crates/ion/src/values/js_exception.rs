// TODO
use crate::Env;
use crate::ToJsUnknown;
use crate::platform::Value;
use crate::values::FromJsValue;
use crate::values::JsValue;
use crate::values::ToJsValue;

#[derive(Clone)]
pub struct JsException {
    pub(crate) value: Value,
    pub(crate) env: Env,
}

impl JsException {
    /// # SAFETY
    ///
    /// Skips checks for type conversion (TODO)
    pub unsafe fn cast_unchecked<T: FromJsValue>(self) -> T {
        T::from_js_value(&self.env, self.value).expect("Failed to cast JsException")
    }

    pub fn cast<T: FromJsValue>(self) -> crate::Result<T> {
        T::from_js_value(&self.env, self.value)
    }
}

impl JsValue for JsException {
    fn value(&self) -> &Value {
        &self.value
    }

    fn env(&self) -> &Env {
        &self.env
    }
}

impl ToJsUnknown for JsException {}

impl FromJsValue for JsException {
    fn from_js_value(
        env: &Env,
        value: Value,
    ) -> crate::Result<Self> {
        Ok(Self { value, env: *env })
    }
}

impl ToJsValue for JsException {
    fn to_js_value(
        _env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(val.value)
    }
}
