use std::sync::{Arc, Condvar, Mutex};
use tokio::sync::Notify;

#[derive(Clone, Default)]
pub struct CompleteSignal {
    inner: Arc<Inner>,
}

impl std::fmt::Debug for CompleteSignal {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("CompleteSignal")
            .finish()
    }
}

struct Inner {
    completed: Mutex<bool>,
    condvar: Condvar,
    notify: Notify,
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            completed: Mutex::new(false),
            condvar: Condvar::new(),
            notify: Notify::new(),
        }
    }
}

impl CompleteSignal {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn done(&self) {
        let mut completed = self.inner.completed.lock().unwrap();
        if !*completed {
            *completed = true;
            self.inner.condvar.notify_all();
            self.inner.notify.notify_waiters();
        }
    }

    pub fn wait(&self) {
        let mut completed = self.inner.completed.lock().unwrap();
        while !*completed {
            completed = self.inner.condvar.wait(completed).unwrap();
        }
    }

    pub async fn wait_async(&self) {
        {
            let completed = self.inner.completed.lock().unwrap();
            if *completed {
                return;
            }
        }

        let notified = self.inner.notify.notified();

        {
            let completed = self.inner.completed.lock().unwrap();
            if *completed {
                return;
            }
        }

        notified.await;
    }

    pub fn is_done(&self) -> bool {
        *self.inner.completed.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[tokio::test]
    async fn test_complete_signal() {
        let sig = CompleteSignal::default();

        thread::spawn({
            let sig = sig.clone();
            move || {
                println!("** Controller WAITING");
                thread::sleep(Duration::from_millis(100));
                println!("** Controller DONE");
                sig.done();
            }
        });

        thread::spawn({
            let sig = sig.clone();
            move || {
                println!("Sync WAITING");
                sig.wait();
                println!("Sync DONE 1");
                sig.wait();
                sig.wait();
                println!("Sync DONE 2");
            }
        });

        println!("Async WAITING");
        sig.wait_async().await;
        println!("Async DONE 1");

        // Subsequent calls after the signal is complete will complete immediately
        sig.wait_async().await;
        sig.wait_async().await;
        println!("Async DONE 2");

        assert!(sig.is_done());
    }
}
