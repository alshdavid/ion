/// Throw a JavaScript exception with the given value
pub fn v8_throw_exception<'a>(
    scope: &mut v8::PinnedRef<'a, v8::HandleScope<'a, ()>>,
    value: v8::Local<v8::Value>,
) {
    scope.throw_exception(value);
}

/// Create and throw a JavaScript Error with the given message
pub fn v8_throw_error<'a>(
    scope: &mut v8::PinnedRef<'a, v8::HandleScope<'a, v8::Context>>,
    message: &str,
) -> crate::Result<()> {
    let Some(msg) = v8::String::new(scope, message) else {
        return Err(crate::Error::ValueCreateError);
    };

    let error = v8::Exception::error(scope, msg);
    scope.throw_exception(error);
    Ok(())
}
