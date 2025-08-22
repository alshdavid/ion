use crate::Env;

/// Convert the handle into v8 value
pub trait ToJsRaw {
    fn into_js_raw(&self) -> v8::Local<'_, v8::Value>;
}

/// Convert a v8 value into a handle
pub trait FromJsRaw {
    fn from_js_raw(
        env: &Env,
        value: v8::Local<'_, v8::Value>,
    ) -> Self;
}

/// Convert a Rust value into a handle
pub trait IntoJsValue<T>: Sized {
    fn into_js_value(
        env: &Env,
        value: T,
    ) -> crate::Result<Self>;
}

/// Convert a handle value into a Rust value
pub trait FromJsValue<T> {
    fn from_js_value(
        &self,
        env: &Env,
    ) -> crate::Result<T>;
}
