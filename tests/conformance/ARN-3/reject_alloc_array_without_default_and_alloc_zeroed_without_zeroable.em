#$ test: compile-fail
#$ rules: ARN-3, ARN-11
# `[ARN-3]` (SP-017, ODR-049) — `alloc_array` needs `Default`, even for a
# `Zeroable` type, and `alloc_zeroed` needs `Zeroable`, even for a type with a
# `Default`: neither stands in for the other. Before SP-017 a `Zeroable` type
# took zero bytes from `alloc_array`, so adding `Default` to one could not
# change what it got, and a proof of `Zeroable` could change a `Default`
# type's.

from std.core import Default

type Small = int in -5..5

struct Seven:
    v: i32

extend Seven implements Default:
    fn default() -> Seven:
        return Seven(7)

fn made[T: Zeroable](frame: Arena, n: int) -> MutSpan[T]:
    return frame.alloc_array[T](n) #$ error[E2040]: `T` does not implement `Default`, which `alloc_array` needs

fn main():
    frame = Arena.with_capacity(1024)
    small = frame.alloc_array[Small](2) #$ error[E2040]: `Small` does not implement `Default`, which `alloc_array` needs
    pointers = frame.alloc_array[*i32](2) #$ error[E2040]: `*i32` does not implement `Default`, which `alloc_array` needs
    sevens = frame.alloc_zeroed[Seven](2) #$ error[E2040]: `Seven` is not `Zeroable`: nothing shows that all-zero bytes are a valid `Seven`
    println(small.len(), pointers.len(), sevens.len())
