#$ test: compile-fail
#$ rules: BRW-8, FN-6
# `[BRW-8]` (ODR-048) — `inner[str]` takes `w` by address, as `inner[T]`
# declares it, because its result points into `w`. A value of type
# `fn(Wrap[str]) -> ref str` would take `w` as a copy (a `Wrap[str]` is a
# view), so the instance is not a value of that type; it is called directly.

struct Wrap[T]:
    item: T

fn inner[T](w: Wrap[T]) -> ref T:
    return ref w.item

fn main():
    _f: fn(Wrap[str]) -> ref str = inner[str] #$ error[E2020]: `inner` is not a value of type `fn(Wrap[str]) -> ref str`: it takes `w` by address
