#$ test: compile-fail
#$ rules: ARN-11, DRV-1
# `[ARN-11]` (ODR-064) — `@derive(Zeroable)` is proved field by field: a
# `ref` field is `E2080`, and an instance over a `ref` is not `Zeroable`. A
# struct that does not derive it is not, whatever its fields: `NonZero`'s
# field is an integer, and zero bytes break its invariant. Deriving it on an
# enum is not built yet.

from std.core import NonZero

@derive(Zeroable)
struct Bad:
    r: ref int #$ error[E2080]: field `r` has type `ref i64`, which is not Zeroable

@derive(Zeroable)
struct Wrap[T]:
    item: T

@derive(Zeroable) #$ error[E0900]: deriving `Zeroable` is not implemented yet
enum Mode:
    Idle

fn main():
    frame = Arena.with_capacity(64)
    a = frame.alloc_zeroed[Wrap[ref int]](1) #$ error[E2040]: `Wrap[ref i64]` is not `Zeroable`
    b = frame.alloc_zeroed[NonZero[i32]](1) #$ error[E2040]: `NonZero[i32]` is not `Zeroable`
    println(a.len(), b.len())
