#$ test: run-pass
#$ rules: FFI-8, THR-14
#$ profiles: debug, development, release
#$ stdout: 1 2 2
#$ stdout: 12 12
# `[FFI-8]`, `[THR-14]` — C's `_Atomic T` is `std.sync.Atomic[T]` when C lays it out as `T` (an
# integer, a `bool` or a pointer): the field and the pointer are Ember's atomics, and C's own
# atomic operations on them are the same ones.

from std.sync import Atomic, MemoryOrder
import c "atomics.h"

fn main():
    counter = c.Counter(hits = Atomic(0), limit = 10)
    first: u32 = 0
    second: u32 = 0
    unsafe:  # SAFETY: `counter` lives for both calls
        first = c.counter_bump(ref_to_ptr(ref mut counter))
        second = c.counter_bump(ref_to_ptr(ref mut counter))
    println(first, second, counter.hits.load(MemoryOrder.Acquire))
    word: Atomic[u32] = Atomic(0)
    word.store(12, MemoryOrder.Release)
    peeked: u32 = 0
    unsafe:  # SAFETY: `word` lives for the call
        peeked = c.word_peek(ref_to_ptr(ref mut word))
    println(word.load(MemoryOrder.Relaxed), peeked)
