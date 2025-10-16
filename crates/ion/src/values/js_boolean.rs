use crate::Env;
use crate::ToJsUnknown;
use crate::platform::sys;
use crate::platform::sys::Value;
use crate::values::FromJsValue;
use crate::values::JsValue;
use crate::values::ToJsValue;

#[derive(Clone)]
pub struct JsBoolean {
    pub(crate) value: Value,
    pub(crate) env: Env,
}

impl JsBoolean {
    pub fn new(
        env: &Env,
        val: bool,
    ) -> crate::Result<Self> {
        let scope = &mut env.scope();
        let boolean = v8::Boolean::new(scope, val);
        Ok(Self {
            value: sys::v8_from_value(boolean),
            env: env.clone(),
        })
    }

    pub fn get_value(&self) -> crate::Result<bool> {
        let Ok(local) = self.value.try_cast::<v8::Boolean>() else {
            return Err(crate::Error::ValueCastError);
        };
        Ok(local.is_true())
    }
}

impl JsValue for JsBoolean {
    fn value(&self) -> &Value {
        &self.value
    }

    fn env(&self) -> &Env {
        &self.env
    }
}

impl ToJsUnknown for JsBoolean {}

impl FromJsValue for JsBoolean {
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

impl ToJsValue for JsBoolean {
    fn to_js_value(
        _env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(val.value)
    }
}

impl ToJsValue for bool {
    fn to_js_value(
        env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(*JsBoolean::new(env, val)?.value())
    }
}

impl Env {
    pub fn create_boolean(
        &self,
        value: bool,
    ) -> crate::Result<JsBoolean> {
        JsBoolean::new(self, value)
    }
}
