#$ test: run-pass
#$ rules: R8, R13
#$ stdout: hi
## R8 (b): the way out E3060's message gives: the view of the payload is used before the line that
## changes the variant.

enum Shape:
    Circle(r: float)
    Label(text: String)

fn main():
    s = Shape.Label(String("hi"))
    if Shape.Label(t) = s:
        println(t)
        s = Shape.Circle(1.0)
