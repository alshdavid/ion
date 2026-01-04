use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct FinalizerRegistry {
    #[allow(clippy::type_complexity)]
    callbacks: Rc<RefCell<HashMap<usize, (v8::Weak<v8::Value>, Box<dyn FnOnce()>)>>>,
    isolate: *mut v8::Isolate,
}

impl FinalizerRegistry {
    pub fn new(isolate: *mut v8::Isolate) -> Self {
        Self {
            callbacks: Default::default(),
            isolate,
        }
    }

    pub fn register(
        &self,
        value: &v8::Local<'_, v8::Value>,
        callback: impl 'static + FnOnce(),
    ) -> usize {
        let mut callback = Box::new(callback);
        let id = callback.as_mut() as *mut _ as usize;

        let weak = v8::Weak::with_guaranteed_finalizer(
            unsafe { &mut *self.isolate },
            value,
            Box::new({
                let callbacks = self.callbacks.clone();
                move || {
                    let mut callbacks = callbacks.borrow_mut();
                    if let Some((_, callback)) = callbacks.remove(&id) {
                        callback();
                    };
                }
            }),
        );

        let mut callbacks = self.callbacks.borrow_mut();
        callbacks.insert(id, (weak, callback));
        id
    }

    pub fn clear(&self) {
        let mut callbacks = self.callbacks.borrow_mut();
        for (_, (_, callback)) in callbacks.drain() {
            callback();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn should_run_drop() -> anyhow::Result<()> {
        let worker = testing::JS_RUNTIME.spawn_worker(JsWorkerOptions {
            resolvers: vec![],
            transformers: vec![],
            extensions: vec![],
        })?;

        let context = worker.create_context()?;
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        context.exec_blocking(move |env| {
            let value = env.create_int32(42)?;

            env.finalizer_registry.register(value.value(), move || {
                tx.send(()).ok();
            });

            env.global_this()?.set_named_property("__global", value)?;

            Ok(())
        })?;

        assert_eq!(rx.len(), 0, "Unexpect GC Notification");

        context.exec_blocking(|env| {
            env.global_this()?.delete_named_property("__global")?;
            Ok(())
        })?;

        // TODO: It appears that the value will only be dropped if the context is dropped
        //       Ideally I want it to be dropped when the value is actually GC'd
        //
        // worker.run_garbage_collection_for_testing()?;
        // assert_eq!(rx.len(), 1, "GC notification not sent");

        drop(context);
        assert_eq!(rx.len(), 1, "GC notification not sent");

        Ok(())
    }
}
