pub fn v8_create_http_request<'a>(
    scope: &mut v8::HandleScope<'a, v8::Context>
) -> v8::Local<'a, v8::Value> {
    let response = v8::Object::new(scope);

    response.into()
}
