use std::ffi::c_void;

use crate::Env;
use crate::Error;
use crate::FromJsValue;
use crate::IntoJsValue;

use super::super::FromJsRaw;
use super::super::ToJsRaw;

pub struct JsString {
    pub(self) handle: *mut c_void,
}

impl JsString {
    fn inner(&self) -> &mut v8::Local<'_, v8::String> {
        unsafe { &mut *(self.handle as *mut v8::Local<'_, v8::String>) }
    }

    pub fn to_string_lossy(
        &self,
        env: &Env,
    ) -> crate::Result<String> {
        self.from_js_value(env)
    }
}

impl Drop for JsString {
    fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.handle as *mut v8::Local<'_, v8::String>) })
    }
}

impl FromJsRaw for JsString {
    fn from_js_raw(
        _env: &Env,
        value: v8::Local<'_, v8::Value>,
    ) -> Self {
        let handle = Box::into_raw(Box::new(value.cast::<v8::String>()));
        JsString {
            handle: handle as _,
        }
    }
}

impl ToJsRaw for JsString {
    fn into_js_raw(&self) -> v8::Local<'_, v8::Value> {
        unsafe { *(self.handle as *mut v8::Local<'_, v8::Value>) }
    }
}

impl<S: AsRef<str>> IntoJsValue<S> for JsString {
    fn into_js_value(
        env: &Env,
        value: S,
    ) -> crate::Result<Self> {
        let Some(value) = v8::String::new(env.context_scope(), value.as_ref()) else {
            return Err(Error::StringCreateError);
        };

        Ok(JsString {
            handle: Box::into_raw(Box::new(value)) as _,
        })
    }
}

impl FromJsValue<String> for JsString {
    fn from_js_value(
        &self,
        env: &Env,
    ) -> crate::Result<String> {
        Ok(self.inner().to_rust_string_lossy(env.context_scope()))
    }
}

impl Env {
    pub fn create_string(
        &self,
        value: impl AsRef<str>,
    ) -> crate::Result<JsString> {
        JsString::into_js_value(self, value)
    }
}
