use std::path::Path;

use flume::Sender;
use flume::bounded;

use crate::Env;
use crate::Error;
use crate::JsUnknown;
use crate::platform::worker::JsWorkerEvent;
use crate::utils::PathExt;
use crate::utils::channel::oneshot;

/// This is a handle to a v8::Context
#[derive(Debug, Clone)]
pub struct JsContext {
    pub(crate) id: usize,
    pub(crate) tx: Sender<JsWorkerEvent>,
}

impl JsContext {
    pub fn exec(
        &self,
        callback: impl 'static + Send + FnOnce(&Env) -> crate::Result<()>,
    ) -> crate::Result<()> {
        if self
            .tx
            .try_send(JsWorkerEvent::Exec {
                id: self.id,
                callback: Box::new(callback),
            })
            .is_err()
        {
            return Err(Error::ExecError);
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
            return Err(Error::ExecError);
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
            return Err(Error::ExecError);
        };
        Ok(ret)
    }

    /// Evaluate script, ignoring return value. If you need the return value
    /// use a variant of [`JsContext::exec`] then run [`Env::eval_script`]
    pub fn eval(
        &self,
        code: impl AsRef<str>,
    ) -> crate::Result<()> {
        let code = code.as_ref().to_string();
        self.exec_blocking(move |env| {
            env.eval_script::<JsUnknown>(code)?;
            Ok(())
        })
    }

    /// Load a file and evaluate it
    pub fn import(
        &self,
        path: impl AsRef<Path>,
    ) -> crate::Result<()> {
        let (tx, rx) = bounded(1);

        self.tx.try_send(JsWorkerEvent::Import {
            id: self.id,
            specifier: path.as_ref().try_to_string()?,
            resolve: tx,
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
            .send(JsWorkerEvent::RequestContextShutdown {
                id: self.id,
                resolve: Some(tx),
            })
            .is_err()
        {
            panic!("Cannot drop JsContext 1")
        };

        if rx.recv().is_err() {
            panic!("Cannot drop JsContext 2")
        }
    }
}
