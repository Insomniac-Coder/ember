#$ test: run-pass
#$ rules: BRW-16, GRM-44
#$ stdout: hi
## design rules: R8, R13
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
