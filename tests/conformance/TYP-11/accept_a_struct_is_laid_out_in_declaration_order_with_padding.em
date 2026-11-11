#$ test: run-pass
#$ rules: TYP-11
#$ stdout: 12 4 12 4
#$ stdout: 6 2
# `[TYP-11]` — a struct's default layout is C's: declaration order, natural alignment, trailing
# padding. `@layout(c)` states it explicitly. `u8, u32, u8` is 1 + 3 + 4 + 1 + 3 = 12 bytes; `u8,
# u8, u16, u16` is 6 bytes with alignment 2.

struct Spread:
    a: u8
    b: u32
    c: u8

@layout(c)
struct Explicit:
    a: u8
    b: u32
    c: u8

struct Packed:
    a: u8
    b: u8
    c: u16
    d: u16

fn main():
    println(mem.size_of[Spread](), mem.align_of[Spread](), mem.size_of[Explicit](), mem.align_of[Explicit]())
    println(mem.size_of[Packed](), mem.align_of[Packed]())
