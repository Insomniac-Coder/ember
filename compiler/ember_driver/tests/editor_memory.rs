//! `[IDE-6]` — the compiler keeps its state in the session (`ember_driver::session::Session`):
//! an editor's edits and queries do not make it grow, and dropping it frees what it held. Every
//! byte this test binary allocates is counted, so the one test here is alone in it.

// A global allocator that counts is the one way to see every byte; it only forwards to `System`.
#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use ember_driver::session::{Position, Session};

#[path = "support/temporary_directory.rs"]
mod test_directories;
use test_directories::temporary_directory;

struct Counting;

static LIVE: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LIVE.fetch_add(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        LIVE.fetch_add(new_size, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static COUNTING: Counting = Counting;

/// Two versions of one file, as an editor's buffer goes back and forth between them: the second
/// has an error the first has not (an unresolvable name, outside `total`'s initializer, whose
/// type an error operand would make unknown [DIA-14]), and they use the same identifiers.
const EDITS: [&str; 2] = [
    "struct Point:\n    x: f64\n    y: f64\n\nfn length(p: Point) -> f64:\n    total = p.x * p.x + p.y * p.y\n    return total\n\nfn main():\n    println(length(Point(3.0, 4.0)))\n",
    "struct Point:\n    x: f64\n    y: f64\n\nfn length(p: Point) -> f64:\n    total = p.x * p.x + p.y * p.y\n    return total\n\nfn main():\n    println(length(Point(3.0, 4.0)) + total)\n",
];

fn round(session: &mut Session, path: &std::path::Path, n: usize) {
    session.set_buffer(path, EDITS[n % 2]);
    let at = Position { line: 6, column: 5 };
    assert_eq!(session.hover(path, at).map(|h| h.text).as_deref(), Some("total: f64"));
    assert!(!session.complete(path, Position { line: 6, column: 15 }).is_empty());
    assert_eq!(session.symbols(path).len(), 3);
    assert_eq!(session.problems(path).len(), n % 2);
}

#[test]
fn a_session_does_not_grow_with_edits_and_frees_everything_when_dropped() {
    let dir = temporary_directory("editor-memory");
    let path = dir.join(format!("edited.{}", ember_branding::SOURCE_EXT));
    std::fs::write(&path, EDITS[0]).expect("the file is written");

    // A first session interns every name the program and the standard library use, which the
    // process keeps (`ember_span::Symbol`, see the notes on `[IDE-6]`); the rest is the session's.
    {
        let mut warm = Session::new();
        round(&mut warm, &path, 0);
        round(&mut warm, &path, 1);
    }
    let before = LIVE.load(Ordering::Relaxed);

    let mut session = Session::new();
    for n in 0..10 {
        round(&mut session, &path, n);
    }
    let after_ten = LIVE.load(Ordering::Relaxed);
    for n in 10..50 {
        round(&mut session, &path, n);
    }
    let after_fifty = LIVE.load(Ordering::Relaxed);
    let held = after_ten - before;
    assert!(held > 100_000, "the session holds a check: {held} bytes");
    // Forty more edits and queries: the same, give or take what one check's sizes vary by.
    assert!(
        after_fifty <= after_ten + held / 10,
        "the session grew from {after_ten} to {after_fifty} bytes over forty edits (it holds {held})"
    );

    session.clear();
    let cleared = LIVE.load(Ordering::Relaxed);
    assert!(cleared < before + held / 10, "a cleared session keeps {} bytes of its {held}", cleared - before);
    drop(session);
    let dropped = LIVE.load(Ordering::Relaxed);
    assert!(dropped < before + 4096, "a dropped session leaves {} bytes", dropped.saturating_sub(before));
}
