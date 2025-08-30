// TODO: Cancellation

use std::time::Duration;

pub use crate::Env;
pub use crate::utils::v8_create_function_from_closure;

pub fn define_set_timeout(env: &Env) {
    let env = env.clone();

    let ctx = env.context();
    let scope = env.context_scope();

    let global_this = ctx.global(scope);

    let js_fn = v8_create_function_from_closure(scope, {
        let env = env.clone();

        move |cb_scope, args, _return_value| {
            let callback = {
                let arg0 = args.get(0).try_cast::<v8::Function>().unwrap();
                v8::Global::new(cb_scope, arg0)
            };

            let duration = {
                let arg1 = args.get(1).try_cast::<v8::Number>().unwrap();
                let a = v8::Local::new(cb_scope, arg1);
                a.int32_value(cb_scope).unwrap()
            };

            env.spawn_async({
                let env = env.clone();

                async move {
                    loop {
                        let callback = callback.clone();

                        crate::utils::sleep::sleep(Duration::from_millis(duration as _)).await;

                        {
                            let scope = &mut env.open_scope();
                            let a = v8::Local::new(scope, callback);
                            let recv = v8::undefined(scope);
                            a.call(scope, recv.into(), &[]);
                        };
                    }
                }
            })
            .unwrap();
        }
    });

    let js_key = v8::String::new(scope, "setInterval").unwrap();
    let js_fn = js_fn.to_local(scope).unwrap();
    global_this.set(scope, js_key.into(), js_fn.into());
}
