use std::ffi::c_void;

use crate::Env;
use crate::FromJsRaw;
use crate::ToJsRaw;

/// JsBuffer is a mutable Vec<u8> that is sharable with the JavaScript runtime across threads
pub struct JsBuffer {
    pub(self) _handle: *mut c_void,
}

impl FromJsRaw for JsBuffer {
    fn from_js_raw(
        _env: &Env,
        _value: v8::Local<'_, v8::Value>,
    ) -> Self {
        todo!()
    }
}

impl ToJsRaw for JsBuffer {
    fn into_js_raw(&self) -> v8::Local<'_, v8::Value> {
        todo!()
    }
}
