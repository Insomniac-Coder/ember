#$ test: compile-fail
#$ rules: BRW-8, FN-6
# `[BRW-8]` (ODR-048) — `inner[str]` takes `w` by address, as `inner[T]`
# declares it, because its result points into `w`. A value of type
# `fn(Wrap[str]) -> ref str` would take `w` as a copy (a `Wrap[str]` is a
# view), so the instance is not a value of that type; it is called directly.

[T]
struct Wrap:
    item: T

[T]
fn inner(w: Wrap[T]) -> ref T:
    return ref w.item

fn main():
    _f: fn(Wrap[StrView]) -> ref StrView = inner[StrView] #$ error[E2020]: `inner` is not a value of type `fn(Wrap[StrView]) -> ref StrView`: it takes `w` by address
