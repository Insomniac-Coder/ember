#$ test: compile-fail
#$ rules: R14, R5
## R14: "`dyn I` as a parameter's type is a read-only view (`ref mut v: dyn I` may change it)"; a
## method whose first line says `changes this` called through it is E3023 (R5).

interface Shape:
    fn size() -> int
    fn grow(by: int) changes this

struct Circle implements Shape:
    r: int

    fn size() -> int:
        return this.r

    fn grow(by: int) changes r:
        this.r += by

fn enlarge(s: dyn Shape):
    s.grow(1)    #$ error[E3023]: `grow` changes the shape (its first line says `changes this`), and `s` only reads it; write `fn enlarge(ref mut s: dyn Shape):` and call it `enlarge(ref mut c)`

fn main():
    c = Circle(r=1)
    enlarge(c)
    println(c.size())
