/// Environment exposes a high level type to interact with the
/// current underlying JavaScript execution context
pub struct Environment<'a, 'b> {
  pub context: v8::Local<'a, v8::Context>,
  pub context_scope: v8::ContextScope<'b, v8::HandleScope<'a>>,
}