#[allow(non_camel_case_types)]
pub type __v8_value = v8::Local<'static, v8::Value>;

pub fn v8_new_value(value: v8::Local<'_, v8::Value>) -> __v8_value {
    unsafe { std::mem::transmute(value)}
}

pub fn v8_from_value<'a>(value: impl Into<v8::Local<'a, v8::Value>>) -> __v8_value {
    unsafe { std::mem::transmute(value.into()) }
}

pub fn v8_into_static_value<'a, V, T>(value: v8::Local<'a, T>) -> v8::Local<'static, V> {
    unsafe { std::mem::transmute(value) }
}
