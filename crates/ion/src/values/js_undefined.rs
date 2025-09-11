use crate::Env;
use crate::ToJsUnknown;
use crate::platform::sys;
use crate::platform::sys::Value;
use crate::values::FromJsValue;
use crate::values::JsValue;
use crate::values::ToJsValue;

#[derive(Clone)]
pub struct JsUndefined {
    pub(crate) value: Value,
    pub(crate) env: Env,
}

impl JsUndefined {
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

impl JsValue for JsUndefined {
    fn value(&self) -> &Value {
        &self.value
    }

    fn env(&self) -> &Env {
        &self.env
    }
}

impl ToJsUnknown for JsUndefined {}

impl FromJsValue for JsUndefined {
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

impl ToJsValue for JsUndefined {
    fn to_js_value(
        _env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(val.value)
    }
}

impl Env {
    pub fn get_undefined(&self) -> crate::Result<JsUndefined> {
        let scope = &mut self.scope();
        JsUndefined::from_js_value(self, sys::v8_from_value(v8::undefined(scope)))
    }
}
