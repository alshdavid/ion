use crate::Env;
use crate::ToJsUnknown;
use crate::platform::sys;
use crate::platform::sys::Value;
use crate::values::FromJsValue;
use crate::values::JsValue;
use crate::values::ToJsValue;

#[derive(Clone)]
pub struct JsNull {
    pub(crate) value: Value,
    pub(crate) env: Env,
}

impl JsNull {
    /// # SAFETY
    ///
    /// Skips checks for type conversion (TODO)
    pub unsafe fn cast_unchecked<T: FromJsValue>(self) -> T {
        T::from_js_value(&self.env, self.value).expect("Failed to cast JsUnknown")
    }

    pub fn cast<T: FromJsValue>(self) -> crate::Result<T> {
        T::from_js_value(&self.env, self.value)
    }

    pub fn type_of(&self) -> String {
        let scope = &mut self.env.scope();
        self.value.type_of(scope).to_rust_string_lossy(scope)
    }
}

impl JsValue for JsNull {
    fn value(&self) -> &Value {
        &self.value
    }

    fn env(&self) -> &Env {
        &self.env
    }
}

impl ToJsUnknown for JsNull {}

impl FromJsValue for JsNull {
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

impl ToJsValue for JsNull {
    fn to_js_value(
        _env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(val.value)
    }
}

impl Env {
    pub fn get_null(&self) -> crate::Result<JsNull> {
        let scope = &mut self.scope();
        JsNull::from_js_value(self, sys::v8_from_value(v8::null(scope)))
    }
}
