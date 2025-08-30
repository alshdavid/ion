use std::ffi::c_void;

use crate::Env;
use crate::FromJsRaw;
use crate::ToJsRaw;

pub struct JsFunction {
    pub(self) _handle: *mut c_void,
}

impl FromJsRaw for JsFunction {
    fn from_js_raw(
        _env: &Env,
        _value: v8::Local<'_, v8::Value>,
    ) -> Self {
        todo!()
    }
}

impl ToJsRaw for JsFunction {
    fn into_js_raw(&self) -> v8::Local<'_, v8::Value> {
        todo!()
    }
}
