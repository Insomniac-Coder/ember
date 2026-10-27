#$ test: compile-fail
#$ rules: R14
## R14: a kind test asks whether a value of an interface or base-class type is a `T`; a `T` that does
## not implement the interface can never be one: E2020 (AN-67 part 6 gives the code, not the
## message; this message is written to R18).

interface Shape:
    fn size() -> int

struct Circle implements Shape:
    r: int

    fn size() -> int:
        return this.r

struct Label:
    text: String

fn radius(s: dyn Shape) -> int:
    if s is Label as l:    #$ error[E2020]: `Label` does not implement `Shape`, so `s` is never a `Label` and this block would never run; test for a kind of `Shape`: `if s is Circle as c:`
        return l.text.len()
    return 0

fn main():
    println(radius(Circle(r=1)))
