#$ test: compile-fail
#$ rules: R14
## R14: "there is no "maybe a `Circle`" value to `match` on"; a choice among kinds is written with
## `if`/`elif`/`else` kind tests (AN-68).

interface Shape:
    fn size() -> int

struct Circle implements Shape:
    r: int

    fn size() -> int:
        return this.r

fn radius(s: dyn Shape) -> int:
    match cast s if Circle:    #$ error[E1010]: a choice among kinds is written with `is`: instead of the `match` line and its two branches, write `if s is Circle as c:` for the `Some(c):` branch and `else:` for the `None:` branch
        Some(c):
            return c.r
        None:
            return 0

fn main():
    println(radius(Circle(r=2)))
