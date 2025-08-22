use super::Context;
use super::ContextFunc;
use super::Environment;

pub struct ContextPoolOptions {
    pub threads: usize,
}

pub struct ContextPool {
    tx_js: crossbeam::channel::Sender<ContextFunc>,
}

impl ContextPool {
    pub fn new(options: &ContextPoolOptions) -> Self {
        let (tx_js, rx_js) = crossbeam::channel::unbounded::<ContextFunc>();

        for i in 0..options.threads {
            let rx_js = rx_js.clone();
            let ctx = Context::new();

            tokio::task::spawn(async move {
                while let Ok(func) = rx_js.recv() {
                    println!("Event for: {}", i);

                    ctx.exec(func);
                }
            });
        }

        Self { tx_js }
    }

    pub fn exec(&self, callback: impl 'static + Send + FnOnce(Environment)) {
        self.tx_js.send(Box::new(callback)).unwrap();
    }

    pub async fn exec_async(&self, callback: impl 'static + Send + FnOnce(Environment)) {
        let (tx_done, rx_done) = tokio::sync::oneshot::channel::<()>();

        self.tx_js.send(Box::new(|env| {
          callback(env);
          tx_done.send(()).unwrap();
        })).unwrap();

        rx_done.await.unwrap()
    }
}
