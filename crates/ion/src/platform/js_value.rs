pub trait ToJsValue {
    type Target: ?Sized;
    fn into_js_value(&self) -> Self::Target;
}

pub trait FromJsValue<T> {
    fn from_js_value(
        isolate: &mut v8::Isolate,
        value: T,
    ) -> Self;
}
