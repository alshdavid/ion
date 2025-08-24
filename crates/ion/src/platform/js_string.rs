use std::ptr::NonNull;

use crate::Env;

use super::FromJsValue;
use super::ToJsValue;

pub struct JsString {
    pub(crate) handle: NonNull<v8::String>,
}

impl ToJsValue for JsString {
    type Target = v8::Local<'static, v8::String>;
    fn into_js_value(&self) -> Self::Target {
        unsafe { std::mem::transmute::<NonNull<v8::String>, Self::Target>(self.handle) }
    }
}

impl FromJsValue<v8::Local<'_, v8::Value>> for JsString {
    fn from_js_value(isolate: &mut v8::Isolate, value: v8::Local<'_, v8::Value>) -> Self {
        let result = value.cast::<v8::String>();
        let result = v8::Global::new(isolate, result);
        let handle = result.into_raw();
        JsString { handle }
    }
}

impl FromJsValue<v8::Local<'_, v8::String>> for JsString {
    fn from_js_value(isolate: &mut v8::Isolate, value: v8::Local<'_, v8::String>) -> Self {
        let result = v8::Global::new(isolate, value);
        let handle = result.into_raw();
        JsString { handle }
    }
}

impl JsString {
    pub fn into_utf8(&self, env: &Env) -> String {
        let mut scope = env.scope();
        let value = self.into_js_value();
        let result = value.to_rust_string_lossy(&mut scope);
        result
    }
}

pub trait JsStringExt {
    fn create_string(&self, value: &str) -> JsString;
}

impl JsStringExt for Env {
    fn create_string(&self, value: &str) -> JsString {
        let mut scope = self.scope();
        let value = v8::String::new(&mut scope, value).unwrap();
        JsString::from_js_value(self.isolate(), value)
    }
}
