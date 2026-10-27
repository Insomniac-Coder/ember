#$ test: run-pass
#$ rules: R14, R4, R5
#$ stdout: 2
## R14: "`ref mut v: dyn I` may change it"; the call says `ref mut` (R4); that `grow` changes the shape
## is worked out from `Circle`'s body, and no first line says it (R5, verdict 86).

interface Shape:
    fn size() -> int
    fn grow(by: int)

struct Circle implements Shape:
    r: int

    fn size() -> int:
        return this.r

    fn grow(by: int):
        this.r += by

fn enlarge(ref mut s: dyn Shape):
    s.grow(1)

fn main():
    c = Circle(r=1)
    enlarge(ref mut c)
    println(c.size())
