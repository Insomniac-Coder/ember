#$ test: compile-fail
#$ rules: LAY-2
#$ profiles: debug
#$ error[E0104]: `@align` takes one alignment
# `[LAY-2]` — `@align(N)` takes a power of two up to 4096, written as a constant; anything else is
# `E0104`, never an alignment rounded to the nearest one or ignored.

const LINE: int = 48

@align(3)  #$ error[E0104]: `@align` takes a power of two up to 4096
struct Three:
    a: u8

@align(8192)  #$ error[E0104]: `@align` takes a power of two up to 4096
struct Huge:
    a: u8

@align(0)  #$ error[E0104]: `@align` takes a power of two up to 4096
struct Zero:
    a: u8

@align(LINE)  #$ error[E0104]: `@align` takes a power of two up to 4096
struct Named:
    a: u8

@align
struct Bare:
    a: u8

@align(4096)
struct Page:
    a: u8

fn main():
    println(size_of[Page]())
