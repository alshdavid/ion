use crate::http_server::HttpEvent;

pub fn v8_create_http_response<'a>(
    scope: &mut v8::HandleScope<'a, v8::Context>,
    tx: &ion::utils::channel::Sender<HttpEvent>,
) -> v8::Local<'a, v8::Value> {
    let response = v8::Object::new(scope);

    // Response.writeHead
    {
        let key = v8::String::new(scope, "writeHead").unwrap();
        let func = ion::utils::v8_create_function_from_closure(scope, {
            let tx = tx.clone();
            move |scope, args, _| {
                let arg0 = args.get(0);
                let status = arg0.uint32_value(scope).unwrap();
                tx.try_send(HttpEvent::WriteHead(status)).unwrap();
            }
        })
        .to_local(scope)
        .unwrap();

        response.set(scope, key.into(), func.into());
    }

    // Response.write
    {
        let key = v8::String::new(scope, "write").unwrap();
        let func = ion::utils::v8_create_function_from_closure(scope, {
            let tx = tx.clone();
            move |scope, args, _| {
                let arg0 = args.get(0);
                let status = arg0.to_rust_string_lossy(scope);
                tx.try_send(HttpEvent::Write(status)).unwrap();
            }
        })
        .to_local(scope)
        .unwrap();

        response.set(scope, key.into(), func.into());
    }

    // Response.end
    {
        let key = v8::String::new(scope, "end").unwrap();
        let func = ion::utils::v8_create_function_from_closure(scope, {
            let tx = tx.clone();
            move |_scope, _args, _| {
                tx.try_send(HttpEvent::End).unwrap();
            }
        })
        .to_local(scope)
        .unwrap();

        response.set(scope, key.into(), func.into());
    }

    response.into()
}
