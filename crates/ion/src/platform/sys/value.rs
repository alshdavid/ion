#[allow(non_camel_case_types)]
pub type Value = v8::Local<'static, v8::Value>;

pub fn v8_from_value<'a>(value: impl Into<v8::Local<'a, v8::Value>>) -> Value {
    unsafe { std::mem::transmute(value.into()) }
}

pub fn v8_into_static_value<'a, V, T>(value: v8::Local<'a, T>) -> v8::Local<'static, V> {
    unsafe { std::mem::transmute(value) }
}

pub fn v8_value_cast<'a, V, T>(value: v8::Local<'a, T>) -> v8::Local<'static, V> {
    unsafe { std::mem::transmute(value) }
}
