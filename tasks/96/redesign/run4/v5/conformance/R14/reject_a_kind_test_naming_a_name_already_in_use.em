#$ test: compile-fail
#$ rules: R14
## R14: the name a kind test gives "is new (E2106)" (AN-68).

interface Shape:
    fn size() -> int

struct Circle implements Shape:
    r: int

    fn size() -> int:
        return this.r

fn radius(s: dyn Shape) -> int:
    if s is Circle as s:    #$ error[E2106]: `s` already names a value (line 14); `as` declares a new name: write `if s is Circle as c:`, and `c.r` for `s.r` in its block
        return s.r
    return 0

fn main():
    println(radius(Circle(r=2)))
