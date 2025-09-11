use crate::Env;
use crate::JsUnknown;
use crate::platform::sys;

pub trait FromJsValue: Sized {
    /// this function called to convert JavaScript values to native rust values
    fn from_js_value(
        env: &Env,
        value: sys::__v8_value,
    ) -> crate::Result<Self>;
}

pub trait JsValue: Sized + FromJsValue {
    fn value(&self) -> &sys::__v8_value;
    fn env(&self) -> &Env;

    fn type_of(&self) -> String {
        let scope = &mut self.env().scope();
        let type_of = self.value().type_of(scope);
        type_of.to_rust_string_lossy(scope)
    }
}

pub trait ToJsValue: Sized {
    /// this function called to convert rust values to JavaScript values
    fn to_js_value(
        env: &Env,
        val: Self,
    ) -> crate::Result<sys::__v8_value>;
}

pub trait ToJsUnknown: Sized + ToJsValue {
    /// this function called to convert JavaScript values into unknown JavaScript values
    fn into_unknown(
        env: &Env,
        val: Self,
    ) -> crate::Result<JsUnknown> {
        Ok(JsUnknown {
            env: env.clone(),
            value: ToJsValue::to_js_value(env, val)?,
        })
    }
}
