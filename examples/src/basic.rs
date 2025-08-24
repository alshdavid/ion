use ion;
use ion::JsString;

static CODE: &str = r#"
  "Hello World"
"#;

pub fn main() {
    // Start the runtime
    ion::platform::initialize_once();
    let jsrt = ion::JsRuntime::new();

    // Evaluate a string, can return any FromJsValue type
    let result: JsString = jsrt.eval(CODE);

    // Open a scope within the isolate and execute some Rust
    jsrt.exec(|env| {
        let result = result.into_utf8(&env);
        println!("{}", result);
    });
}
