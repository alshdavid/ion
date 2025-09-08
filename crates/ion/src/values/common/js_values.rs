use crate::Env;
use crate::JsUnknown;
use crate::platform::Value;

pub trait FromJsValue: Sized {
    /// this function called to convert JavaScript values to native rust values
    fn from_js_value(
        env: &Env,
        value: Value,
    ) -> crate::Result<Self>;
}

pub trait JsValue: Sized + FromJsValue {
    fn value(&self) -> &Value;
    fn env(&self) -> &Env;

    fn type_of(&self) -> String {
        let scope = &mut self.env().scope();
        let type_of = self.value().inner().type_of(scope);
        type_of.to_rust_string_lossy(scope)
    }
}

pub trait ToJsValue: Sized {
    /// this function called to convert rust values to JavaScript values
    fn to_js_value(
        env: &Env,
        val: Self,
    ) -> crate::Result<Value>;
}

pub trait ToJsUnknown: Sized + ToJsValue {
    /// this function called to convert JavaScript values into unknown JavaScript values
    fn into_unknown(
        env: &Env,
        val: Self,
    ) -> crate::Result<JsUnknown> {
        Ok(JsUnknown {
            env: *env,
            value: ToJsValue::to_js_value(env, val)?,
        })
    }
}
