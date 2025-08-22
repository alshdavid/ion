use flume::Sender;
use flume::bounded;

use crate::Error;
use crate::JsWorkerEvent;
use crate::utils::channel::oneshot;

use super::Env;

/// This is a handle to a v8::Context
#[derive(Debug, Clone)]
pub struct JsContext {
    pub(super) id: usize,
    pub(super) tx: Sender<JsWorkerEvent>,
}

impl JsContext {
    pub fn exec(
        &self,
        callback: impl 'static + Send + FnOnce(Env) -> crate::Result<()>,
    ) -> crate::Result<()> {
        if self
            .tx
            .try_send(JsWorkerEvent::Exec(self.id, Box::new(callback)))
            .is_err()
        {
            return Err(Error::ExecError);
        };
        Ok(())
    }

    pub async fn exec_async(
        &self,
        callback: impl 'static + Send + FnOnce(Env) -> crate::Result<()>,
    ) -> crate::Result<()> {
        let (tx, rx) = bounded(1);

        self.exec(move |env| {
            callback(env)?;
            tx.send(()).unwrap();
            Ok(())
        })?;

        if rx.recv_async().await.is_err() {
            return Err(Error::ExecError);
        };

        Ok(())
    }

    pub fn exec_blocking(
        &self,
        callback: impl 'static + Send + FnOnce(Env) -> crate::Result<()>,
    ) -> crate::Result<()> {
        let (tx, rx) = bounded(1);

        self.exec(move |env| {
            callback(env)?;
            tx.try_send(()).unwrap();
            Ok(())
        })?;

        if rx.recv().is_err() {
            return Err(Error::ExecError);
        };
        Ok(())
    }
}

impl Drop for JsContext {
    fn drop(&mut self) {
        let (tx, rx) = oneshot();
        if self
            .tx
            .send(JsWorkerEvent::ShutdownContext(self.id, tx))
            .is_err()
        {
            // Do nothing, worker is shut down
        };
        rx.recv().unwrap();
    }
}
