#$ test: run-pass
#$ rules: FFI-11, FFI-8
#$ profiles: debug, development, release
#$ stdout: 7 7 7
#$ stdout: 10 4
#$ stdout: 1
# `[FFI-11]` — `borrowed one` on a pointer to a struct the header never defines (`[FFI-8]`: an
# `extern type`, only ever behind the pointer): a `const Counter*` is a `ref Counter`, a `Counter*`
# a `ref mut Counter` (an incomplete type takes no parameter mode, so not `mut counter: Counter`),
# `nullable` an `Option` of either; a result `from(counter)` borrows the argument. No `unsafe`.

import c "./handles.c" with (overlay = "overlays/handles.em") as h

# The box frees its counter, with `counter_free`, when it is dropped here.
fn count() -> i32:
    counter = h.counter_new(5)
    h.counter_add(ref mut counter, 2)
    same: ref h.Counter = h.counter_same(counter)
    println(h.counter_get(counter), h.counter_get(same), h.counter_get_or(Some(counter), 0))
    again: ref mut h.Counter = h.counter_same_mut(ref mut counter)
    h.counter_add(again, 2)
    h.counter_add_if(None, 100)
    h.counter_add_if(Some(ref mut counter), 1)
    return h.counter_get(counter)

fn main():
    println(count(), h.counter_get_or(None, 4))
    println(h.counter_freed())
