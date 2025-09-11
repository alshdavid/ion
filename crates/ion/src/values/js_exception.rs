// TODO
use crate::Env;
use crate::ToJsUnknown;
use crate::platform::sys;
use crate::values::FromJsValue;
use crate::values::JsValue;
use crate::values::ToJsValue;

#[derive(Clone)]
pub struct JsException {
    pub(crate) value: sys::__v8_value,
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
    fn value(&self) -> &sys::__v8_value {
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
        value: sys::__v8_value,
    ) -> crate::Result<Self> {
        Ok(Self {
            value,
            env: env.clone(),
        })
    }
}

impl ToJsValue for JsException {
    fn to_js_value(
        _env: &Env,
        val: Self,
    ) -> crate::Result<sys::__v8_value> {
        Ok(val.value)
    }
}
