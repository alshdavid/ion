// For now use globalThis.handler, eventually use module exports
pub fn get_handler<'a>(
    scope: &mut v8::HandleScope<'a, v8::Context>
) -> v8::Local<'a, v8::Function> {
    let context = scope.get_current_context();
    let global_this = context.global(scope);

    let key = v8::String::new(scope, "handler").unwrap();
    global_this
        .get(scope, key.into())
        .unwrap()
        .cast::<v8::Function>()
}
