#$ test: run-pass
#$ rules: LAY-2, TYP-11
#$ profiles: debug, development, release
#$ stdout: 24 8 0 8 16 | 24 8 0 8 16
#$ 6 2 | 6 2 0 2 4
# `[LAY-2]`, `[TYP-11]` — `@layout(c)` is the C layout every struct has by default: fields in
# declaration order, each at its natural alignment, and trailing padding to the struct's own
# alignment. Writing it changes nothing.

@layout(c)
struct Stated:
    a: u8
    b: f64
    c: u16

struct Implied:
    a: u8
    b: f64
    c: u16

@layout(c)
enum Kind:
    Small(u8)
    Large(u16, u8)

@layout(c)
struct Bytes:
    a: u8
    b: u16
    c: u8

fn main():
    println(size_of[Stated](), align_of[Stated](), offset_of[Stated](a), offset_of[Stated](b), offset_of[Stated](c), "|", size_of[Implied](), align_of[Implied](), offset_of[Implied](a), offset_of[Implied](b), offset_of[Implied](c))
    println(size_of[Kind](), align_of[Kind](), "|", size_of[Bytes](), align_of[Bytes](), offset_of[Bytes](a), offset_of[Bytes](b), offset_of[Bytes](c))
