use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(1);
pub struct IdGenerator;

impl IdGenerator {

    pub fn next() -> usize {
        COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    pub fn current() -> usize {
        COUNTER.load(Ordering::SeqCst)
    }
}