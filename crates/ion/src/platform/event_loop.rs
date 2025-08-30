use std::sync::Arc;
use std::thread;

use flume::bounded;
use futures::task::LocalSpawnExt;

use crate::Error;
use crate::executor::LocalPool;
use crate::executor::LocalSpawner;
use crate::executor::ThreadNotify;
use crate::executor::wait_for_wake;

pub struct EventLoop {
    pool: LocalPool,
    spawner: LocalSpawner,
    thread_notify: Arc<ThreadNotify>,
}

impl EventLoop {
    pub(crate) fn new(thread_notify: Arc<ThreadNotify>) -> Self {
        let pool = LocalPool::new();
        let spawner = pool.spawner();
        Self {
            pool,
            spawner,
            thread_notify,
        }
    }

    pub fn start_waker_thread(callback: impl 'static + Send + Fn()) -> Arc<ThreadNotify> {
        let (tx_thread_notify, rx_thread_notify) = bounded::<Arc<ThreadNotify>>(1);
        thread::spawn(move || {
            let thread_notify = ThreadNotify::new();
            tx_thread_notify.send(thread_notify.clone()).unwrap();

            loop {
                wait_for_wake(&thread_notify);
                callback();
            }
        });
        rx_thread_notify.recv().unwrap()
    }

    /// Spawns a task that polls the given future to completion.
    pub fn spawn_local(
        &self,
        task: impl Future<Output = ()> + 'static,
    ) -> crate::Result<()> {
        if self.spawner.spawn_local(task).is_err() {
            return Err(Error::TaskSpawnError);
        }

        Ok(())
    }

    /// Run all tasks in the pool to completion.
    pub fn run_to_completion(&mut self) -> crate::Result<()> {
        self.pool.run(&self.thread_notify);
        Ok(())
    }

    /// Runs all tasks in the pool and returns if no more progress can be made on any task.
    pub fn run_until_stalled(&mut self) -> crate::Result<()> {
        self.pool.run_until_stalled(&self.thread_notify);
        Ok(())
    }
}
