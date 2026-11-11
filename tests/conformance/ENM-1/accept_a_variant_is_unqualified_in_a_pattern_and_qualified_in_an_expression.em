#$ test: run-pass
#$ rules: ENM-1
#$ stdout: 3
# `[ENM-1]` — inside a pattern whose scrutinee type is known, a variant may be written unqualified
# (`Circle(r)`); in an expression it is qualified (`Shape.Circle(1.0)`).

enum Shape:
    Circle(i32)
    Square(i32)

fn main():
    s = Shape.Circle(3)
    match s:
        Circle(r):
            println(r)
        Square(w):
            println(w)
