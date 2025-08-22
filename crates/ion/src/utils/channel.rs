pub use flume::*;

pub fn channel<T>() -> (flume::Sender<T>, flume::Receiver<T>) {
    flume::unbounded()
}

pub fn oneshot<T>() -> (flume::Sender<T>, flume::Receiver<T>) {
    flume::bounded(1)
}
