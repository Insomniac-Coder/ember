#$ test: run-pass
#$ rules: THR-1, THR-2, THR-10, CLO-16, FFI-22, WK-12
#$ stdout: 0
## design rules: R15, R12, R11
## Verdict 114 (2026-10-28, gap map Q-E): a class object handed whole to another thread while a
## `weakref` link on this thread points at it goes, and the link here then finds it ended (Rust's
## `Rc::try_unwrap`). The hand-over reads the weak count; with a link, the contents move to a new block
## for the other thread and the old one, marked ended, stays here until `w` ends. So `if w exists as p:`
## is skipped, and the thread's `names.items.len()` gives 0. Until verdict 114 this program was E7015
## (`reject_an_object_a_weakref_points_at_handed_to_another_thread.em`).

import std.thread

class Names:
    items: Array[String] = []

fn main():
    names = Names()
    w = weakref names
    h = thread.spawn(fn() owns names: names.items.len())
    if w exists as p:
        println(p.items.len())
    println(h.join())
