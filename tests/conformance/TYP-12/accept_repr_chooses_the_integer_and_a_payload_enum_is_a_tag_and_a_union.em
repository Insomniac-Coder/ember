#$ test: run-pass
#$ rules: TYP-12
#$ stdout: 1 2 4 8
#$ 16 8
#$ 5 -1
## `[TYP-12]` - `@repr(u8)` and friends choose the integer a unit-only enum is stored as; a payload enum is a tag
## plus a union of its variants' payloads (an `i64` payload and a one-byte tag: 16 bytes with padding).

@repr(u8)
enum A:
    X
    Y

@repr(u16)
enum B:
    X
    Y

@repr(i32)
enum C:
    X = -1
    Y = 5

@repr(u64)
enum D:
    X
    Y

enum Shape:
    Circle(r: i64)
    Square(side: i64)

enum Tiny:
    One(b: u8)
    Two

fn main():
    println(size_of[A](), size_of[B](), size_of[C](), size_of[D]())
    println(size_of[Shape](), align_of[Shape]())
    println(C.Y as i32, C.X as i32)
