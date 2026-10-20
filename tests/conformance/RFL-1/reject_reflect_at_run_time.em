#$ test: compile-fail
#$ rules: RFL-1
## `[RFL-1]` — a description is made while compiling; `reflect[T]()` is not available at run time.

from std.ser import reflect

struct Point:
    x: f64

fn main():
    d = reflect[Point]()  #$ error[E6010]: `reflect` describes a type while compiling
    println(d.size)
