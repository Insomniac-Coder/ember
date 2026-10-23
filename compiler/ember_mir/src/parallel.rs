//! `[BLD-7]` — the front end's work that takes one body at a time (lowering, and the analyses that
//! check or summarise one body), run on the driver's threads (`-j`).
//!
//! Each body is taken by one thread, which reports into a sink of its own; the bodies' results and
//! diagnostics then come back in the bodies' order, so what a build makes and reports, and in what
//! order, is the same on any number of threads. A body's work reads only what every body's reads
//! (the checked program, the type table, the program's contracts) and writes only its own result.

use std::ops::Range;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use ember_diag::Sink;

static JOBS: AtomicUsize = AtomicUsize::new(1);

/// The number of threads the per-body analyses use: the driver's `-j` (default the number of
/// physical cores).
pub fn set_jobs(jobs: usize) {
    JOBS.store(jobs.max(1), Ordering::Relaxed);
}

pub fn jobs() -> usize {
    JOBS.load(Ordering::Relaxed)
}

/// How many items a thread takes at a time: enough runs per thread that one long body does not
/// leave the others idle, few enough that taking one costs nothing.
fn run_length(items: usize, jobs: usize) -> usize {
    (items / (jobs * 8)).max(1)
}

/// `f` of each of `items`, each reporting into a sink of its own, on `jobs()` threads. The results
/// in the items' order, each with the range of `sink` its diagnostics took there.
pub fn each<I: Sync, T: Send>(items: &[I], sink: &mut Sink, f: impl Fn(&I, &mut Sink) -> T + Sync) -> Vec<(T, Range<usize>)> {
    each_on(jobs(), items, sink, f)
}

/// As [`each`], on `jobs` threads.
pub fn each_on<I: Sync, T: Send>(jobs: usize, items: &[I], sink: &mut Sink, f: impl Fn(&I, &mut Sink) -> T + Sync) -> Vec<(T, Range<usize>)> {
    let jobs = jobs.min(items.len());
    if jobs <= 1 {
        return items
            .iter()
            .map(|item| {
                let start = sink.mark();
                let result = f(item, sink);
                (result, start..sink.mark())
            })
            .collect();
    }
    let run = run_length(items.len(), jobs);
    let next = AtomicUsize::new(0);
    let done: Vec<Vec<(usize, T, Sink)>> = std::thread::scope(|scope| {
        let threads: Vec<_> = (0..jobs)
            .map(|_| {
                scope.spawn(|| {
                    let mut out = Vec::new();
                    loop {
                        let start = next.fetch_add(run, Ordering::Relaxed);
                        if start >= items.len() {
                            return out;
                        }
                        for index in start..(start + run).min(items.len()) {
                            let mut own = Sink::new();
                            let result = f(&items[index], &mut own);
                            out.push((index, result, own));
                        }
                    }
                })
            })
            .collect();
        threads.into_iter().map(|thread| thread.join().unwrap_or_else(|panic| std::panic::resume_unwind(panic))).collect()
    });
    merge(done.into_iter().flatten().collect(), items.len(), sink)
}

/// As [`each`], for an analysis that changes the item it checks.
pub fn each_mut<I: Send, T: Send>(items: &mut [I], sink: &mut Sink, f: impl Fn(&mut I, &mut Sink) -> T + Sync) -> Vec<(T, Range<usize>)> {
    each_mut_on(jobs(), items, sink, f)
}

/// As [`each_mut`], on `jobs` threads.
pub fn each_mut_on<I: Send, T: Send>(jobs: usize, items: &mut [I], sink: &mut Sink, f: impl Fn(&mut I, &mut Sink) -> T + Sync) -> Vec<(T, Range<usize>)> {
    let jobs = jobs.min(items.len());
    if jobs <= 1 {
        return items
            .iter_mut()
            .map(|item| {
                let start = sink.mark();
                let result = f(item, sink);
                (result, start..sink.mark())
            })
            .collect();
    }
    let count = items.len();
    let run = run_length(count, jobs);
    let runs = Mutex::new(items.chunks_mut(run).enumerate());
    let done: Vec<Vec<(usize, T, Sink)>> = std::thread::scope(|scope| {
        let threads: Vec<_> = (0..jobs)
            .map(|_| {
                scope.spawn(|| {
                    let mut out = Vec::new();
                    loop {
                        let taken = runs.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).next();
                        let Some((number, chunk)) = taken else { return out };
                        for (offset, item) in chunk.iter_mut().enumerate() {
                            let mut own = Sink::new();
                            let result = f(item, &mut own);
                            out.push((number * run + offset, result, own));
                        }
                    }
                })
            })
            .collect();
        threads.into_iter().map(|thread| thread.join().unwrap_or_else(|panic| std::panic::resume_unwind(panic))).collect()
    });
    merge(done.into_iter().flatten().collect(), count, sink)
}

/// The threads' results put back in the items' order, their diagnostics into `sink` in that order.
fn merge<T>(done: Vec<(usize, T, Sink)>, count: usize, sink: &mut Sink) -> Vec<(T, Range<usize>)> {
    let mut slots: Vec<Option<(T, Sink)>> = (0..count).map(|_| None).collect();
    for (index, result, own) in done {
        slots[index] = Some((result, own));
    }
    slots
        .into_iter()
        .map(|slot| {
            let (result, own) = slot.expect("every item was checked");
            let start = sink.mark();
            sink.absorb(own);
            (result, start..sink.mark())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ember_diag::Diagnostic;
    use ember_span::Span;

    /// The same results and the same diagnostics, in the same order, on one thread and on many.
    #[test]
    fn many_threads_report_what_one_does_in_the_same_order() {
        let items: Vec<u32> = (0..500).collect();
        let check = |item: &u32, sink: &mut Sink| {
            for k in 0..(item % 3) {
                sink.emit(Diagnostic::warning(ember_diag::codes::W5054, Span::DUMMY, format!("{item}/{k}")));
            }
            item * 2
        };
        // On threads given, not `set_jobs`: the other tests of this process share that.
        let run = |jobs: usize| {
            let mut sink = Sink::new();
            let results = each_on(jobs, &items, &mut sink, check);
            let mut changed = items.clone();
            let mut sink_mut = Sink::new();
            let results_mut = each_mut_on(jobs, &mut changed, &mut sink_mut, |item, sink| {
                *item += 1;
                check(item, sink)
            });
            let messages = |sink: &Sink| sink.diagnostics().iter().map(|d| d.message.clone()).collect::<Vec<_>>();
            (results, messages(&sink), results_mut, messages(&sink_mut), changed)
        };
        let one = run(1);
        let many = run(7);
        assert_eq!(one.0, many.0);
        assert_eq!(one.1, many.1);
        assert_eq!(one.2, many.2);
        assert_eq!(one.3, many.3);
        assert_eq!(one.4, many.4);
    }
}
