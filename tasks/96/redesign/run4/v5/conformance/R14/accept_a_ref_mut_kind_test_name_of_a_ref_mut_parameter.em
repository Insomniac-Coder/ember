#$ test: run-pass
#$ rules: R14, R4
#$ stdout: 2
## R14: "`ref mut v: dyn I` may change it", and the kind test's `ref mut c` names the caller's circle.

interface Shape:
    fn size() -> int

struct Circle implements Shape:
    r: int

    fn size() -> int:
        return this.r

fn widen(ref mut s: dyn Shape):
    if s is Circle as ref mut c:
        c.r += 1

fn main():
    circle = Circle(r=1)
    widen(ref mut circle)
    println(circle.r)
