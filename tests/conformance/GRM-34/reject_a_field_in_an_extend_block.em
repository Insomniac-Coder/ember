#$ test: compile-fail
#$ rules: GRM-34, TYP-11
## `[GRM-34]` — an `extend` block adds methods, constants and associated types; a field in one is `E0100`,
## whose help writes it in the type's own declaration.

struct P:
    x: int = 0

extend P:
    y: int = 0    #$ error[E0100]: an `extend` block adds no field
                  #$ help: write `y: int = 0` in `struct P:`
    fn get() -> int:
        return this.x

fn main():
    p = P()
    println(p.get())
