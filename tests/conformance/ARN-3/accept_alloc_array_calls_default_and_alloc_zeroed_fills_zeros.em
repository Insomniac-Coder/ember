#$ test: run-pass
#$ rules: ARN-3, ARN-11, TYP-17
# `[ARN-3]` (SP-017, ODR-049) — `alloc_array[T](n)` initialises each element
# with `T.default()`; `alloc_zeroed[T](n)` fills with zero bytes, for a
# `Zeroable` `T`. A range type that holds 0 is `Zeroable` and has no
# `Default`, so it takes `alloc_zeroed`. A generic body may call either under
# the matching bound (D-343: a `T: Default` bound was not enough).

from std.core import Default

type Small = int in -5..5

struct Seven:
    v: i32

extend Seven implements Default:
    fn default() -> Seven:
        return Seven(7)

fn made[T: Default](frame: Arena, n: int) -> MutSpan[T]:
    return frame.alloc_array[T](n)

fn zeros[T: Zeroable](frame: Arena, n: int) -> MutSpan[T]:
    return frame.alloc_zeroed[T](n)

fn main():
    frame = Arena.with_capacity(1024)
    sevens = frame.alloc_array[Seven](2)
    small = frame.alloc_zeroed[Small](2)
    first: int = small[1]
    generic = made[Seven](frame, 3)
    bytes = zeros[u8](frame, 4)
    floats = frame.alloc_array[f64](2)
    println(sevens[0].v, sevens[1].v, first, generic[2].v, bytes[3], floats[1])
#$ stdout: 7 7 0 7 0 0.0
#$ assert-c: contains("ember_arena_alloc_zeroed")
