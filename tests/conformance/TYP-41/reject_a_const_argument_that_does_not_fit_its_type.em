#$ test: compile-fail
#$ rules: TYP-41
## `[TYP-41]` rule 1 — a const argument is evaluated as its parameter's type
## evaluates; one that does not fit overflows, `E6004`, where it is written.

fn small[const N: u8]() -> int:
    return 1

struct Tiny[const N: u8]:
    x: int

fn main():
    println(small[300]())    #$ error[E6004]: the const argument `300` for `N` does not fit `u8`
    t = Tiny[256](1)    #$ error[E6004]: the const argument `256` for `N` does not fit `u8`
    println(t.x)
