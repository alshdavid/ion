use std::io::Read;
use std::sync::Arc;

use ion::*;
use parking_lot::Mutex;

pub fn main() -> anyhow::Result<()> {
    let runtime = JsRuntime::initialize_once(JsRuntimeOptions {
        v8_args: vec![],
        resolvers: vec![ion::resolvers::relative()],
        transformers: vec![
            ion::transformers::json(),
            ion::transformers::ts(),
            ion::transformers::tsx(),
        ],
        extensions: vec![
            ion::extensions::event_target(),
            ion::extensions::console(),
            ion::extensions::set_timeout(),
            ion::extensions::set_interval(),
            ion::extensions::test(),
            ion::extensions::global_this(),
        ],
    })?;

    let worker = runtime.spawn_worker(JsWorkerOptions::default())?;
    let ctx = worker.create_context()?;

    ctx.exec_blocking(define_text_decoder)?;

    ctx.exec_blocking(|env| {
        let buffer = Arc::new(Mutex::new("Hello World".as_bytes()));
        let mut reader = JsObject::new(env)?;

        reader.set_named_property(
            "read",
            JsFunction::new(env, {
                let buffer = Arc::clone(&buffer);
                move |env, ctx| {
                    let mut js_arr = ctx.arg::<JsArray>(0)?;

                    let mut view: Vec<u8> = std::iter::repeat(0)
                        .take(js_arr.length()? as usize)
                        .collect();

                    let mut buffer = buffer.lock();

                    match buffer.read(&mut view) {
                        Ok(bytes_read) => {
                            for i in 0..bytes_read {
                                js_arr.set_element(i as u32, env.create_uint32(view[i] as u32)?)?;
                            }
                            return env.create_uint32(bytes_read as u32);
                        }
                        Err(_err) => todo!(),
                    }
                }
            })?,
        )?;

        env.global_this()?.set_named_property("myReader", reader)?;
        Ok(())
    })?;

    ctx.eval(
        r#"
        const { myReader } = globalThis

        const result = []

        const buffer = [0,0]
        while (true) {
            console.log('Seeking')
            const bytes = myReader.read(buffer)
            for (let i = 0; i < bytes; i++) {
                result.push(buffer[i])
            }
            if (bytes < buffer.length) {
                break
            }
        }

        console.log(result)
        console.log((new TextDecoder).decode(result))
    "#,
    )?;

    Ok(())
}

fn define_text_decoder(env: &Env) -> ion::Result<()> {
    let ctor = JsFunction::new(env, |env, ctx| {
        let mut this = ctx.this().cast::<JsObject>()?;

        this.set_named_property(
            "decode",
            JsFunction::new(env, |env, ctx| {
                let arr = ctx.arg::<JsArray>(0)?;

                let mut buf = Vec::<u8>::new();
                for i in 0..arr.length()? {
                    let entry = arr.get_element::<JsNumber>(i)?.unwrap();
                    let value = entry.get_u32()? as u8;
                    buf.push(value);
                }

                JsString::new(env, String::from_utf8(buf)?)
            })?,
        )?;

        Ok(this)
    })?;

    env.global_this()?.set_named_property("TextDecoder", ctor)?;

    Ok(())
}
