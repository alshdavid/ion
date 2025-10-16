/// Throw a JavaScript exception with the given value
pub fn v8_throw_exception(
    scope: &mut v8::HandleScope<v8::Context>,
    value: v8::Local<v8::Value>,
) {
    scope.throw_exception(value);
}

/// Create and throw a JavaScript Error with the given message
pub fn v8_throw_error(
    scope: &mut v8::HandleScope<v8::Context>,
    message: &str,
) -> crate::Result<()> {
    let Some(msg) = v8::String::new(scope, message) else {
        return Err(crate::Error::ValueCreateError);
    };

    let error = v8::Exception::error(scope, msg);
    scope.throw_exception(error);
    Ok(())
}

// /// Create and throw a JavaScript TypeError with the given message
// pub fn v8_throw_type_error(
//     scope: &mut v8::HandleScope<v8::Context>,
//     message: &str,
// ) -> crate::Result<()> {
//     let Some(msg) = v8::String::new(scope, message) else {
//         return Err(crate::Error::ValueCreateError);
//     };

//     let error = v8::Exception::type_error(scope, msg);
//     scope.throw_exception(error);
//     Ok(())
// }

// /// Create and throw a JavaScript RangeError with the given message
// pub fn v8_throw_range_error(
//     scope: &mut v8::HandleScope<v8::Context>,
//     message: &str,
// ) -> crate::Result<()> {
//     let Some(msg) = v8::String::new(scope, message) else {
//         return Err(crate::Error::ValueCreateError);
//     };

//     let error = v8::Exception::range_error(scope, msg);
//     scope.throw_exception(error);
//     Ok(())
// }

// /// Create and throw a JavaScript ReferenceError with the given message
// pub fn v8_throw_reference_error(
//     scope: &mut v8::HandleScope<v8::Context>,
//     message: &str,
// ) -> crate::Result<()> {
//     let Some(msg) = v8::String::new(scope, message) else {
//         return Err(crate::Error::ValueCreateError);
//     };

//     let error = v8::Exception::reference_error(scope, msg);
//     scope.throw_exception(error);
//     Ok(())
// }

// /// Create and throw a JavaScript SyntaxError with the given message
// pub fn v8_throw_syntax_error(
//     scope: &mut v8::HandleScope<v8::Context>,
//     message: &str,
// ) -> crate::Result<()> {
//     let Some(msg) = v8::String::new(scope, message) else {
//         return Err(crate::Error::ValueCreateError);
//     };

//     let error = v8::Exception::syntax_error(scope, msg);
//     scope.throw_exception(error);
//     Ok(())
// }
