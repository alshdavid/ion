use std::pin::Pin;
use std::thread::JoinHandle;
use std::thread::{self};

use flume::Sender;
use flume::unbounded;

use crate::utils::channel::oneshot;

pub(crate) enum BackgroundTaskManagerEvent {
    ExecFut {
        fut: Pin<Box<dyn 'static + Send + Sync + Future<Output = crate::Result<()>>>>,
    },
}

pub struct BackgroundTaskManager {
    tx: Sender<BackgroundTaskManagerEvent>,
    #[allow(unused)]
    handle: JoinHandle<crate::Result<()>>,
}

impl BackgroundTaskManager {
    pub fn new() -> crate::Result<Self> {
        let (tx, rx) = unbounded::<BackgroundTaskManagerEvent>();

        let (bg_tx, bg_rx) = oneshot::<crate::Result<()>>();
        let handle: JoinHandle<crate::Result<()>> = thread::spawn({
            move || {
                let Ok(runtime) = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(num_cpus::get_physical())
                    .enable_all()
                    .build()
                else {
                    bg_tx
                        .send(Err(crate::Error::BackgroundThreadError))
                        .expect("Unable to start background thread");

                    return Err(crate::Error::BackgroundThreadError);
                };

                bg_tx
                    .send(Ok(()))
                    .expect("Unable to start background thread");

                runtime.block_on(async move {
                    while let Ok(event) = rx.recv_async().await {
                        match event {
                            BackgroundTaskManagerEvent::ExecFut { fut } => {
                                tokio::task::spawn(fut);
                            }
                        }
                    }
                    Ok(())
                })
            }
        });
        bg_rx.recv()??;

        Ok(Self { tx, handle })
    }

    pub fn spawn(
        &self,
        fut: impl 'static + Send + Sync + Future<Output = crate::Result<()>>,
    ) -> crate::Result<()> {
        Ok(self
            .tx
            .try_send(BackgroundTaskManagerEvent::ExecFut { fut: Box::pin(fut) })?)
    }

    pub fn spawn_then(
        &self,
        fut: impl 'static + Send + Sync + Future<Output = crate::Result<()>>,
    ) -> crate::Result<()> {
        Ok(self
            .tx
            .try_send(BackgroundTaskManagerEvent::ExecFut { fut: Box::pin(fut) })?)
    }
}
