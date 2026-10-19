#$ test: run-pass
#$ rules: LAY-2
#$ profiles: debug, development, release
#$ assert-c: contains("typedef uint16_t em_Port;")
#$ stdout: 1 2 4 8 1 2 4 8
#$ 2 16 16
#$ 443 65535 -300 -5000000000
# `[LAY-2]` — `@repr(T)` makes `T` an enum's discriminant type, for each of the eight integer
# types: a unit-only enum is that integer, and a payload enum's tag has that type, with its
# payload placed after it at the payload's own alignment. Discriminants convert with `as` across
# the whole range of the type.

@repr(u8)
enum A8:
    X

@repr(u16)
enum Port:
    Http = 80
    Https = 443
    Top = 65535

@repr(u32)
enum A32:
    X

@repr(u64)
enum A64:
    X

@repr(i8)
enum B8:
    X

@repr(i16)
enum Offset:
    Back = -300
    Ahead = 300

@repr(i32)
enum B32:
    X

@repr(i64)
enum Wide:
    Far = -5000000000
    Near = 0

@repr(u8)
enum Small:
    Byte(u8)
    Empty

@repr(u64)
enum Tagged:
    Byte(u8)
    Empty

@repr(u16)
enum Spaced:
    Word(u64)
    Empty

fn main():
    println(size_of[A8](), size_of[Port](), size_of[A32](), size_of[A64](), size_of[B8](), size_of[Offset](), size_of[B32](), size_of[Wide]())
    println(size_of[Small](), size_of[Tagged](), size_of[Spaced]())
    println(Port.Https as u16, Port.Top as u16, Offset.Back as i16, Wide.Far as i64)
