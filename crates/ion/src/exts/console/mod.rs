pub use crate::Env;
pub use crate::utils::v8_create_function_from_closure;

pub fn define_console(env: &Env) {
    let scope = &mut env.open_scope();
    let cs = env.context();
    let global_this = cs.global(scope);

    let js_str_console_key = v8::String::new(scope, "console").unwrap();
    let js_obj_console = v8::Object::new(scope);

    // console.log
    {
        let js_fn_log_key = v8::String::new(scope, "log").unwrap();
        let js_fn_log = v8::Function::builder(
            |scope: &mut v8::HandleScope,
             args: v8::FunctionCallbackArguments,
             _rv: v8::ReturnValue| {
                let arg0 = args.get(0);
                let s = arg0.try_cast::<v8::String>().unwrap();
                let s = s.to_rust_string_lossy(scope);
                println!("{:?}", s);
            },
        )
        .build(scope)
        .unwrap();
        js_obj_console.set(scope, js_fn_log_key.into(), js_fn_log.into());
    }

    global_this.set(scope, js_str_console_key.into(), js_obj_console.into());
}
