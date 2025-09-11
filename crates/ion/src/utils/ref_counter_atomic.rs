use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

/// Simple single threaded reference counter
#[derive(Debug, Clone)]
pub struct AtomicRefCounter(Arc<AtomicUsize>);

impl Default for AtomicRefCounter {
    fn default() -> Self {
        Self::new(1)
    }
}

impl AtomicRefCounter {
    pub fn new(start: usize) -> Self {
        Self(Arc::new(AtomicUsize::new(start)))
    }

    pub fn inc(&self) -> usize {
        self.0.fetch_add(1, Ordering::Relaxed)
    }

    pub fn dec(&self) -> bool {
        let previous = self.0.fetch_sub(1, Ordering::Relaxed);
        previous == 1
    }

    pub fn count(&self) -> usize {
        self.0.load(Ordering::Relaxed)
    }
}
