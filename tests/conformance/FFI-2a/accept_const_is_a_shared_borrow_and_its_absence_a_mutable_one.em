#$ test: run-pass
#$ rules: FFI-2a
#$ profiles: debug, development, release
#$ stdout: 4 5
#$ stdout: 0 0 0
# `[FFI-2a]` — derived facts need no `unsafe` and are never written: `const int*` is a read-only
# borrow (a `Span[i32]`), `int*` a mutable one (a `MutSpan[i32]`), so the overlay's entries say
# only what C cannot: the count, and the result's lifetime, which is never derived.

import c "buffers.h" with (overlay = "overlays/buffers.em")

fn main():
    items: Array[i32] = [4, 5, 6]
    println(c.buffers_first(items.as_span()), c.buffers_pick(items.as_span()))
    c.buffers_zero(items.as_mut_span())
    println(items[0], items[1], items[2])
