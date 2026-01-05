use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

static NEXT_THREAD_ID: AtomicUsize = AtomicUsize::new(1);

thread_local! {
    static THREAD_ID: usize = NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed);
}

pub fn thread_id() -> usize {
    THREAD_ID.with(|&id| id)
}
