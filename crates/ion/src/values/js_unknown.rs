use std::ffi::c_void;

use crate::Env;

use super::super::FromJsRaw;
use super::super::ToJsRaw;

pub struct JsUnknown {
    pub(self) handle: *mut c_void,
}

// impl JsUnknown {
//     fn inner(&self) -> &mut v8::Local<'_, v8::Value> {
//         unsafe { &mut *(self.handle as *mut v8::Local<'_, v8::Value>) }
//     }
// }

impl Drop for JsUnknown {
    fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.handle as *mut v8::Local<'_, v8::Value>) })
    }
}

impl FromJsRaw for JsUnknown {
    fn from_js_raw(
        _env: &Env,
        value: v8::Local<'_, v8::Value>,
    ) -> Self {
        let handle = Box::into_raw(Box::new(value.cast::<v8::Value>()));
        JsUnknown {
            handle: handle as _,
        }
    }
}

impl ToJsRaw for JsUnknown {
    fn into_js_raw(&self) -> v8::Local<'_, v8::Value> {
        unsafe { *(self.handle as *mut v8::Local<'_, v8::Value>) }
    }
}
