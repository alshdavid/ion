// use std::{thread, time::Duration};

// use ion;

// static CODE: &str = r#"
//   "Hello World"
// "#;

pub fn main() -> anyhow::Result<()> {
    // let runtime = ion::platform::initialize_once()?;

    // {
    //     let worker = runtime.spawn_worker()?;

    //     {
    //         let ctx = worker.create_context()?;
    //         ctx.exec_blocking(|env| {
    //             let scope = env.scope();

    //             let value = v8_eval_code(scope, "2");
    //             let value = value.try_cast::<v8::Number>().unwrap();
    //             let out = value.int32_value(scope).unwrap();

    //             println!("Returned: {}", out);
    //         });
    //     };

    //     {
    //         let ctx = worker.create_context()?;
    //         ctx.exec_blocking(|env| {
    //             let scope = env.scope();

    //             let value = v8_eval_code(scope, "2");
    //             let value = value.try_cast::<v8::Number>().unwrap();
    //             let out = value.int32_value(scope).unwrap();

    //             println!("Returned: {}", out);
    //         });
    //     };
    // };

    // {
    //     let worker = runtime.spawn_worker()?;

    //     {
    //         let ctx = worker.create_context()?;
    //         ctx.exec_blocking(|env| {
    //             let value = v8_eval_code(env.scope(), "2");
    //             let value = value.try_cast::<v8::Number>().unwrap();
    //             let out = value.int32_value(env.scope()).unwrap();

    //             println!("Returned: {}", out);
    //         });
    //     };

    //     {
    //         let ctx = worker.create_context()?;
    //         ctx.exec_blocking(|env| {
    //             let scope = env.scope();

    //             let value = v8_eval_code(scope, "2");
    //             let value = value.try_cast::<v8::Number>().unwrap();
    //             let out = value.int32_value(scope).unwrap();

    //             println!("Returned: {}", out);
    //         });
    //     };
    // };

    // let h = std::thread::spawn({
    //     let runtime = runtime.clone();

    //     move || -> anyhow::Result<()> {
    //         let worker = runtime.spawn_worker()?;
    //         let ctx = worker.create_context()?;
    //         ctx.exec_blocking(|env| {
    //             let scope = &mut v8::HandleScope::new(env.context_scope());

    //             let value = v8_eval_code(scope, "1");
    //             let value = value.try_cast::<v8::Number>().unwrap();
    //             let out = value.int32_value(scope).unwrap();

    //             println!("Returned: {}", out);
    //         });
    //         Ok(())
    //     }
    // });

    // let h2 = std::thread::spawn({
    //     let runtime = runtime.clone();

    //     move || -> anyhow::Result<()> {
    //         let worker = runtime.spawn_worker()?;
    //         let ctx = worker.create_context()?;
    //         ctx.exec_blocking(|env| {
    //             let scope = &mut v8::HandleScope::new(env.context_scope());

    //             let value = v8_eval_code(scope, "2");
    //             let value = value.try_cast::<v8::Number>().unwrap();
    //             let out = value.int32_value(scope).unwrap();

    //             println!("Returned: {}", out);
    //         });
    //         Ok(())
    //     }
    // });

    // let h3 = std::thread::spawn({
    //     let runtime = runtime.clone();

    //     move || -> anyhow::Result<()> {
    //         let worker = runtime.spawn_worker()?;
    //         let ctx = worker.create_context()?;
    //         ctx.exec_blocking(|env| {
    //             let scope = &mut v8::HandleScope::new(env.context_scope());

    //             let value = v8_eval_code(scope, "3");
    //             let value = value.try_cast::<v8::Number>().unwrap();
    //             let out = value.int32_value(scope).unwrap();

    //             println!("Returned: {}", out);
    //         });
    //         Ok(())
    //     }
    // });

    // // println!("{:?}", runtime);
    // // // println!("{:?}", worker);
    // // // println!("{:?}", ctx);
    // // // drop(runtime);
    // drop(h.join());
    // drop(h2.join());
    // drop(h3.join());
    Ok(())
}

pub fn v8_eval_code<'a, S: AsRef<str>>(
    scope: &mut v8::HandleScope<'a>,
    code: S,
) -> v8::Local<'a, v8::Value> {
    let code = v8::String::new(scope, code.as_ref()).unwrap();
    let script = v8::Script::compile(scope, code, None).unwrap();
    let value = script.run(scope).unwrap();
    value
}
