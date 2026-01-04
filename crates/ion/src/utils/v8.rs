pub fn v8_create_string<'a>(
    scope: &mut v8::HandleScope<'a, v8::Context>,
    s: impl AsRef<str>,
) -> crate::Result<v8::Local<'a, v8::String>> {
    let Some(value) = v8::String::new(scope, s.as_ref()) else {
        return Err(crate::Error::ValueCreateError);
    };

    Ok(value)
}

pub fn v8_create_undefined<'a>(
    scope: &mut v8::HandleScope<'a, v8::Context>
) -> crate::Result<v8::Local<'a, v8::Value>> {
    Ok(v8::undefined(scope).into())
}
