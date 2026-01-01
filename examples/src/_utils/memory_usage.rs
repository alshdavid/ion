use std::sync::Arc;

use memory_stats::memory_stats;
use parking_lot::Mutex;

#[derive(Default)]
pub struct MemoryUsageCounter(Arc<Mutex<isize>>);

impl std::fmt::Debug for MemoryUsageCounter {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let mut previous = self.0.lock();
        let current = Self::get_memory_usage_mb();

        let result = if current > *previous {
            write!(
                f,
                "Memory Usage: {}mb (+{}mb)",
                current,
                current - *previous
            )
        } else if current == *previous {
            write!(f, "Memory Usage: {}mb", current,)
        } else {
            write!(f, "Memory Usage: {}mb ({}mb)", current, current - *previous)
        };

        (*previous) = current;
        result
    }
}

impl MemoryUsageCounter {
    fn get_memory_usage_mb() -> isize {
        if let Some(usage) = memory_stats() {
            let b = usage.physical_mem;
            let kb = b / 1000;
            let mb = kb / 1000;
            mb as isize
        } else {
            panic!("Couldn't get the current memory usage :(");
        }
    }
}
