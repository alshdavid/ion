use flume::Sender;
use flume::bounded;

use crate::Env;
use crate::platform::worker::JsWorkerEvent;

pub struct AsyncEnv {
    pub(crate) tx: Sender<JsWorkerEvent>,
    pub(crate) realm_id: usize,
}

impl AsyncEnv {
    pub fn exec(
        &self,
        callback: impl 'static + Send + FnOnce(&Env) -> crate::Result<()>,
    ) -> crate::Result<()> {
        if self
            .tx
            .try_send(JsWorkerEvent::Exec {
                id: self.realm_id,
                callback: Box::new(callback),
            })
            .is_err()
        {
            return Err(crate::Error::ExecError);
        };
        Ok(())
    }

    pub async fn exec_async<Return: 'static + Send + Sync>(
        &self,
        callback: impl 'static + Send + FnOnce(&Env) -> crate::Result<Return>,
    ) -> crate::Result<Return> {
        let (tx, rx) = bounded(1);

        self.exec(move |env| Ok(tx.try_send(callback(env)?)?))?;

        let Ok(ret) = rx.recv_async().await else {
            return Err(crate::Error::ExecError);
        };
        Ok(ret)
    }

    pub fn exec_blocking<Return: Send + Sync + 'static>(
        &self,
        callback: impl 'static + Send + FnOnce(&Env) -> crate::Result<Return>,
    ) -> crate::Result<Return> {
        let (tx, rx) = bounded::<Return>(1);

        self.exec(move |env| Ok(tx.try_send(callback(env)?)?))?;

        let Ok(ret) = rx.recv() else {
            return Err(crate::Error::ExecError);
        };
        Ok(ret)
    }
}
