// A15 workaround: scoped threads (Rust 1.63+) plus an atomic counter.
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

pub fn count_words(texts: &Vec<String>) -> usize {
    let total = AtomicUsize::new(0);
    thread::scope(|s| {
        for t in texts {
            let total = &total;
            s.spawn(move || { total.fetch_add(t.split_whitespace().count(), Ordering::Relaxed); });
        }
    });
    total.into_inner()
}
