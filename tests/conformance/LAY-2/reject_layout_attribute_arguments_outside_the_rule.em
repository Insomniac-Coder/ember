#$ test: compile-fail
#$ rules: LAY-2
#$ profiles: debug
#$ error[E0104]: `@layout` takes `c`
#$ error[E0104]: `@packed` takes no arguments
#$ error[E0104]: `@repr` names the discriminant's integer type
#$ error[E0104]: `@repr` names the discriminant's integer type
#$ error[E0104]: `@repr` names the discriminant's integer type
#$ error[E0104]: `@repr` does not apply to a struct
# `[LAY-2]` — `@layout` names `c`, `@packed` takes nothing, and `@repr` names one of the eight
# integer types `u8`…`u64`, `i8`…`i64` (`int` is `i64`); anything else is `E0104`. `@repr` gives
# an enum's discriminant type and nothing else: on a struct it is `E0104` (ODR-139).

@layout(rust)
struct Reordered:
    a: u8

@packed(1)
struct One:
    a: u8

@repr(usize)
enum Pointer:
    A

@repr(u128)
enum Wide:
    A

@repr(f32)
enum Float:
    A

@repr(u8)
struct Bytes:
    a: u8

@repr(int)
enum Plain:
    A

fn main():
    println(size_of[Plain]())
