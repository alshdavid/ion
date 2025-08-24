use std::ptr::NonNull;

use super::FromJsValue;
use super::ToJsValue;

pub struct JsUnknown {
    pub(crate) handle: NonNull<v8::Value>,
}

impl ToJsValue for JsUnknown {
    type Target = v8::Local<'static, v8::Value>;

    fn into_js_value(&self) -> Self::Target {
        unsafe { std::mem::transmute::<NonNull<v8::Value>, Self::Target>(self.handle) }
    }
}

impl FromJsValue<v8::Local<'_, v8::Value>> for JsUnknown {
    fn from_js_value(
        isolate: &mut v8::Isolate,
        value: v8::Local<'_, v8::Value>,
    ) -> Self {
        let result = v8::Global::new(isolate, value);
        let handle = result.into_raw();
        JsUnknown { handle }
    }
}
