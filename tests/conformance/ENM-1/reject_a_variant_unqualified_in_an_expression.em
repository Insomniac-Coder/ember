#$ test: compile-fail
#$ rules: ENM-1
# `[ENM-1]` — in an expression a variant is qualified unless imported.

enum Shape:
    Circle(i32)
    Square(i32)

fn main():
    s = Circle(3)                   #$ error[E1010]: cannot find `Circle` in this scope
    println(1)
