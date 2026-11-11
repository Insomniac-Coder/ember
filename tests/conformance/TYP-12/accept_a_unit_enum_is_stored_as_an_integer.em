#$ test: run-pass
#$ rules: TYP-12, ENM-3
#$ stdout: 1 1 2
# `[TYP-12]` — a unit-only enum is stored as an integer (`@repr(u8)` and friends choose it; the
# default is the smallest signed type that fits).

enum Color:
    Red
    Green
    Blue

@repr(u16)
enum Wide:
    A
    B

fn main():
    println(mem.size_of[Color](), mem.size_of[Color](), mem.size_of[Wide]())
