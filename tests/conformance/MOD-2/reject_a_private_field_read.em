#$ test: compile-fail
#$ rules: MOD-2
## A field is private unless it says otherwise.

from support.shapes import make

fn main():
    b = make(3, 4)
    println(b.height)     #$ error[E1020]: `height` is private to `support.shapes.Box2`'s module
