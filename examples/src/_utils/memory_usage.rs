use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use memory_stats::memory_stats;
use parking_lot::Mutex;
use serde::Serialize;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn get_sample() -> usize {
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[derive(Debug, Serialize)]
pub struct MemoryUsageReport {
    pub sample: usize,
    pub units: String,
    pub value: isize,
    pub change: isize,
}

impl MemoryUsageReport {
    pub fn json(&self) -> String {
        serde_json::to_string(&self).unwrap().to_string()
    }
}

#[derive(Default)]
pub struct MemoryUsageCounter(Arc<Mutex<isize>>);

#[allow(dead_code)]
impl MemoryUsageCounter {
    pub fn megabytes(&self) -> MemoryUsageReport {
        let mut previous = self.0.lock();
        let current = Self::get_memory_usage_mb();

        let change = if current > *previous {
            current - *previous
        } else if current == *previous {
            0
        } else {
            current - *previous
        };

        (*previous) = current;

        MemoryUsageReport {
            sample: get_sample(),
            units: "mb".to_string(),
            value: current,
            change,
        }
    }

    pub fn kilobytes(&self) -> MemoryUsageReport {
        let mut previous = self.0.lock();
        let current = Self::get_memory_usage_kb();

        let change = if current > *previous {
            current - *previous
        } else if current == *previous {
            0
        } else {
            -(current - *previous)
        };

        (*previous) = current;

        MemoryUsageReport {
            sample: get_sample(),
            units: "kb".to_string(),
            value: current,
            change,
        }
    }

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

    fn get_memory_usage_kb() -> isize {
        if let Some(usage) = memory_stats() {
            let b = usage.physical_mem;
            let kb = b / 1000;
            kb as isize
        } else {
            panic!("Couldn't get the current memory usage :(");
        }
    }
}

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
