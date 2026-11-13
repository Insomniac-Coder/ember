# Clause BRW-14.1: "a loop or pattern name that views its value (one under `for x in e:` or `match
# e:`) and a name opened with `exists as` (`[GRM-44]`) or named by a kind test (`[DSP-4]`) stand for
# what (...) the loop, the opening or the test gave, and keep it for their whole scope: `=` on one is
# `E3023`"
#$ test: compile-fail
#$ rules: BRW-14, CTL-1, GRM-13, GRM-44, DSP-4
# The spec gives message texts only for parameters: codes only here.

interface Shape:
    fn area() -> int

struct Circle:
    r: int

extend Circle implements Shape:
    fn area() -> int:
        return 3 * this.r * this.r

fn main():
    xs = [1, 2]
    for x in xs:
        x = 5    #$ error[E3023]
    o: Option[int] = Some(1)
    match o:
        Some(v):
            v = 2    #$ error[E3023]
        None:
            pass
    if o exists as w:
        w = 3    #$ error[E3023]
    s: owns dyn Shape = owns Circle(r=1)
    if s is Circle as c:
        c = Circle(r=2)    #$ error[E3023]
