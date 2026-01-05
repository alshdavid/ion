use std::sync::Arc;

use flume::Sender;
use flume::bounded;

use crate::Env;
use crate::Error;
use crate::JsUnknown;
use crate::platform::callback_registry::CallbackRegistry;
use crate::platform::worker::JsWorkerEvent;
use crate::utils::channel::oneshot;

/// This is a handle to a v8::Context
#[derive(Debug, Clone)]
pub struct JsContext {
    pub(crate) callback_registry: Arc<CallbackRegistry>,
    pub(crate) id: usize,
    pub(crate) tx: Sender<JsWorkerEvent>,
}

impl JsContext {
    pub fn exec(
        &self,
        callback: impl 'static + Send + FnOnce(&Env) -> crate::Result<()>,
    ) -> crate::Result<()> {
        let span = tracing::Span::current();
        if self
            .tx
            .try_send(JsWorkerEvent::Exec {
                id: self.id,
                callback: Box::new(callback),
                span,
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
        specifier: impl AsRef<str>,
    ) -> crate::Result<()> {
        let specifier = specifier.as_ref().to_string();
        self.exec_blocking(move |env| env.import(specifier))
    }

    /// Wait for the context to complete all activity
    pub fn join_blocking(self) -> crate::Result<()> {
        if !self.callback_registry.worker_handle_active() {
            return Ok(());
        }

        self.callback_registry
            .context_handle_set_status(&self.id, false);

        let (tx, rx) = oneshot();
        self.callback_registry
            .add_context_shutdown_callback(self.id.clone(), move || tx.try_send(()).unwrap());

        drop(self.tx.send(JsWorkerEvent::ContextHandleDeactivated {
            id: self.id.clone(),
        }));

        if rx.recv().is_err() {
            panic!("Cannot drop JsContext 2")
        }

        Ok(())
    }

    /// Wait for the context to complete all activity
    pub async fn join_async(&self) -> crate::Result<()> {
        self.callback_registry
            .context_handle_set_status(&self.id, false);
        self.tx
            .send(JsWorkerEvent::ContextHandleDropped {
                id: self.id.clone(),
            })
            .unwrap();
        Ok(())
    }
}
