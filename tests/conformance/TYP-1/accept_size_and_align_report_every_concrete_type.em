#$ test: run-pass
#$ rules: TYP-1
#$ stdout: 1
#$ 4
#$ 8
#$ 16
#$ 16
#$ 0
#$ 8
#$ 4
#$ 16
#$ 8
#$ 1
#$ 4
## `[TYP-1]` - every concrete type has a size and an alignment that `size_of[T]()` and `align_of[T]()` report:
## scalars, a struct with padding, a view, a span, a `StrView`, a unit-only enum and a tuple.

struct Padded:
    a: u8
    b: i32

enum Colour:
    Red
    Green

fn main():
    println(size_of[u8]())
    println(size_of[char]())
    println(size_of[ref int]())
    println(size_of[Span[int]]())
    println(size_of[StrView]())
    println(size_of[void]())
    println(size_of[Padded]())
    println(align_of[Padded]())
    println(align_of[u128]())
    println(size_of[(i32, i32)]() * 1)
    println(size_of[Colour]())
    println(align_of[f32]())
