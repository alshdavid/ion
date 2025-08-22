use std::ffi::c_void;

use crate::Env;
use crate::FromJsRaw;
use crate::ToJsRaw;

pub struct JsObject {
    pub(self) _handle: *mut c_void,
}

impl FromJsRaw for JsObject {
    fn from_js_raw(
        _env: &Env,
        _value: v8::Local<'_, v8::Value>,
    ) -> Self {
        todo!()
    }
}

impl ToJsRaw for JsObject {
    fn into_js_raw(&self) -> v8::Local<'_, v8::Value> {
        todo!()
    }
}
