use std::ffi::c_void;

use crate::Env;
use crate::FromJsRaw;
use crate::ToJsRaw;

pub struct JsArray {
    pub(self) _handle: *mut c_void,
}

impl FromJsRaw for JsArray {
    fn from_js_raw(
        _env: &Env,
        _value: v8::Local<'_, v8::Value>,
    ) -> Self {
        todo!()
    }
}

impl ToJsRaw for JsArray {
    fn into_js_raw(&self) -> v8::Local<'_, v8::Value> {
        todo!()
    }
}
