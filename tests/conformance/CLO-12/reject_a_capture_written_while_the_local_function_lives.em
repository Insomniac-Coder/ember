#$ test: compile-fail
#$ rules: CLO-12, CLO-2, BRW-1
## `[CLO-12]` — a local function borrows what it captures from its
## declaration to its last use, as a lambda would: `x` cannot change between.

fn main():
    x = 1
    fn get() -> int:
        return x
    x = 2    #$ error[E3021]: `x` cannot be written while it is borrowed
    println(get())
