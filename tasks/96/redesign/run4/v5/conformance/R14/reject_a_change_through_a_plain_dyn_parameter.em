#$ test: compile-fail
#$ rules: R14, R5
## R14: "`dyn I` as a parameter's type is a read-only view (`ref mut v: dyn I` may change it)"; R5
## (verdict 86): "A method that may change its object is not called through a name that only reads it
## (E3023)", what `grow` changes being worked out from every implementation the program has (here
## `Circle`'s).

interface Shape:
    fn size() -> int
    fn grow(by: int)

struct Circle implements Shape:
    r: int

    fn size() -> int:
        return this.r

    fn grow(by: int):
        this.r += by

fn enlarge(s: dyn Shape):
    s.grow(1)    #$ error[E3023]: `grow` changes the shape (`Circle`'s `grow` adds to `this.r`, line 19), and `s` only reads it; write `fn enlarge(ref mut s: dyn Shape):` in place of line 21, and call it `enlarge(ref mut c)` in place of line 26

fn main():
    c = Circle(r=1)
    enlarge(c)
    println(c.size())
