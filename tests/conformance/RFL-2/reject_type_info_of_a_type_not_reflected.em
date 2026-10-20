#$ test: compile-fail
#$ rules: RFL-2
## `[RFL-2]` — only a `@reflect` type keeps a description for run time.

from std.ser import type_info

struct Plain:
    x: int

fn main():
    println(type_info[Plain]().name)  #$ error[E2040]: `Plain` does not implement `Reflect`
