use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;

use super::platform::platform_once_init;
use super::Environment;

pub type ContextFunc = Box<dyn 'static + Send + FnOnce(Environment)>;

/// Context is a JavaScript execution context that runs in a
/// dedicated thread. It represents an isolated environment to
/// interact with JavaScript.
pub struct Context {
  tx: Sender<ContextFunc>,
}

impl Context {
  pub fn new() -> Self {
    platform_once_init();

    let (tx, rx) = channel::<ContextFunc>();

    std::thread::spawn(move || {
      let mut isolate = v8::Isolate::new(v8::CreateParams::default());

      while let Ok(callback) = rx.recv() {
        let mut handle_scope = v8::HandleScope::new(&mut isolate);
        let context = v8::Context::new(&mut handle_scope, Default::default());
        let context_scope = v8::ContextScope::new(&mut handle_scope, context);

        let env = Environment {
          context,
          context_scope,
        };

        callback(env);
      }
    });
    Context { tx }
  }

  pub fn exec(
    &self,
    callback: impl 'static + Send + FnOnce(Environment),
  ) {
    self.tx.send(Box::new(callback)).unwrap();
  }
}
