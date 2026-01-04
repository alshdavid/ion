use crate::Env;
use crate::ToJsUnknown;
use crate::platform::sys::Value;
use crate::utils::v8::v8_create_undefined;
use crate::values::FromJsValue;
use crate::values::JsValue;
use crate::values::ToJsValue;

#[derive(Clone)]
pub struct JsUnknown {
    pub(crate) value: Value,
    pub(crate) env: Env,
}

impl JsUnknown {
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

impl JsValue for JsUnknown {
    fn value(&self) -> &Value {
        &self.value
    }

    fn env(&self) -> &Env {
        &self.env
    }
}

impl ToJsUnknown for JsUnknown {}

impl FromJsValue for JsUnknown {
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

impl ToJsValue for JsUnknown {
    fn to_js_value(
        _env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(val.value)
    }
}

impl ToJsValue for () {
    fn to_js_value(
        env: &Env,
        _val: Self,
    ) -> crate::Result<Value> {
        let scope = &mut env.scope();
        let local = v8_create_undefined(scope)?;
        Ok(local)
    }
}
