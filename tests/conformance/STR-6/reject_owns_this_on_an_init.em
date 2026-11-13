#$ test: compile-fail
#$ rules: STR-6, CLS-2
## `[STR-6]` - `owns this` on an `init` is `E2108`: `init` builds the object, so it cannot take it over.

struct A:
    x: int

    fn init(x: int) owns this:    #$ error[E2108]: `init` builds the object, so it cannot take it over; write `fn init(x: int):`
        this.x = x

fn main():
    println(A(1).x)
