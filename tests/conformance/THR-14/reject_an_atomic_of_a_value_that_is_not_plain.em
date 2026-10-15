#$ test: compile-fail
#$ rules: THR-14
#$ error[E2020]: an `Atomic` holds an integer, a `bool` or a raw pointer, not `String`
#$ error[E1010]: `std.sync.Atomic[bool]` has no method named `fetch_add`
#$ error[E2020]: expected `std.sync.MemoryOrder`, found `an integer`
# `[THR-14]` — `Atomic[T]` exists for the integer types, `bool` and raw pointers; arithmetic is
# for integers; the order is a `MemoryOrder`.

from std.sync import Atomic

fn main():
    s = Atomic("x".to_string())
    f = Atomic(true)
    f.fetch_add(true)
    a = Atomic(1)
    a.load(1)
