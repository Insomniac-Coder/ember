#$ test: compile-fail
#$ rules: FN-2a
## design rules: R14, R4
## R14: the name a kind test gives may be `ref mut` "where the function may change the place"; `s`, a
## plain `dyn Shape` parameter, is "a read-only view" (R14), so a `ref mut` name of it is E3027.

interface Shape:
    fn size() -> int

struct Circle implements Shape:
    r: int

    fn size() -> int:
        return this.r

fn widen(s: dyn Shape):
    if s is Circle as ref mut c:    #$ error[E3027]: `s` only reads the caller's shape, so `c`, a name for it, cannot be `ref mut`; write `fn widen(ref mut s: dyn Shape):` and call it `widen(ref mut circle)`
        c.r += 1

fn main():
    circle = Circle(r=1)
    widen(circle)
    println(circle.r)
