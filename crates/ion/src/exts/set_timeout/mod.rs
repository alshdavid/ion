use std::cell::LazyCell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

use flume::Sender;

pub use crate::Env;
use crate::utils::channel::oneshot;
pub use crate::utils::v8_create_function_from_closure;

thread_local! {
    static TIMEOUTS: LazyCell<RefCell<(
        HashMap<u32, Sender<()>>,
        u32,
    )>> = Default::default();
}

fn insert_timeout(sig: Sender<()>) -> u32 {
    TIMEOUTS.with(|c| {
        let mut state = c.borrow_mut();
        state.1 += 1;
        let id = state.1.clone();
        state.0.insert(id.clone(), sig);
        id
    })
}

fn remove_timeout(id: &u32) -> Option<Sender<()>> {
    TIMEOUTS.with(|c| {
        let mut state = c.borrow_mut();
        state.0.remove(id)
    })
}

pub fn define_set_timeout(env: &Env) {
    let env = env.clone();

    let ctx = env.context();
    let scope = env.context_scope();

    let global_this = ctx.global(scope);

    // setTimeout
    {
        let js_key = v8::String::new(scope, "setTimeout").unwrap();
        let js_fn = v8_create_function_from_closure(scope, {
            let env = env.clone();

            move |cb_scope, args, mut return_value| {
                let callback = {
                    let arg0 = args.get(0).try_cast::<v8::Function>().unwrap();
                    Box::new(v8::Global::new(cb_scope, arg0))
                };

                let duration = {
                    let arg1 = args.get(1).try_cast::<v8::Number>().unwrap();
                    let a = v8::Local::new(cb_scope, arg1);
                    a.int32_value(cb_scope).unwrap()
                };

                let (tx, rx) = oneshot();
                let id = insert_timeout(tx);

                env.spawn_async_local({
                    let env = env.clone();

                    async move {
                        env.sleep(Duration::from_millis(duration as _)).await;
                        drop(remove_timeout(&id));
                        if !rx.is_empty() {
                            return;
                        }

                        {
                            let scope = &mut env.open_scope();
                            let a = v8::Local::new(scope, *callback);
                            let recv = v8::undefined(scope);
                            a.call(scope, recv.into(), &[]);
                        };
                    }
                })
                .unwrap();

                let js_id = v8::Integer::new_from_unsigned(cb_scope, id);
                return_value.set(js_id.into());
            }
        });

        let js_fn = js_fn.to_local(scope).unwrap();

        global_this.set(scope, js_key.into(), js_fn.into());
    }

    // clearTimeout
    {
        let js_key = v8::String::new(scope, "clearTimeout").unwrap();
        let js_fn = v8_create_function_from_closure(scope, {
            move |cb_scope, args, _return_value| {
                let id = {
                    let arg0 = args.get(0).cast::<v8::Number>();
                    arg0.uint32_value(cb_scope).unwrap()
                };

                let Some(tx) = remove_timeout(&id) else {
                    panic!("No timeout for: {}", id)
                };

                tx.try_send(()).unwrap();
            }
        })
        .to_local(scope)
        .unwrap();

        global_this.set(scope, js_key.into(), js_fn.into());
    }
}
