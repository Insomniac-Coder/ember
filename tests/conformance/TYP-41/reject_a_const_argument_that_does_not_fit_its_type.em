#$ test: compile-fail
#$ rules: TYP-41
## `[TYP-41]` rule 1 — a const argument is evaluated as its parameter's type
## evaluates; one that does not fit overflows, `E6004`, where it is written.

[const N: u8]
fn small() -> int:
    return 1

[const N: u8]
struct Tiny:
    x: int

fn main():
    println(small[300]())    #$ error[E6004]: the const argument `300` for `N` does not fit `u8`
    t = Tiny[256](1)    #$ error[E6004]: the const argument `256` for `N` does not fit `u8`
    println(t.x)
