#$ test: compile-fail
#$ rules: CLI-19, GRM-45
## `[CLI-19]` — a construct this compiler parses and does not build is `E0900`, by name, once: the
## kind test `if s is Circle as c:` (`[GRM-45]`). Its name is bound to nothing known, so the block's
## uses of it are not refused again.

interface Shape:
    fn area() -> int

struct Circle implements Shape:
    r: int = 1

    fn area() -> int:
        return 3 * this.r * this.r

fn main():
    s: owns dyn Shape = owns Circle(r=2)
    if s is Circle as c:    #$ error[E0900]: a kind test that names what it finds is not built by this compiler yet
        println(c.r)
