#$ test: compile-fail
#$ rules: R8, R13
#$ help: println(t)
## R8 (b): a view into an enum's payload ends when the enum changes its variant, a reshape that frees the
## payload (E3060 at a later use).

enum Shape:
    Circle(r: float)
    Label(text: String)

fn main():
    s = Shape.Label(String("hi"))
    if Shape.Label(t) = s:
        s = Shape.Circle(1.0)
        println(t)    #$ error[E3060]: `s = Shape.Circle(1.0)` (line 14) changes `s` to another variant, which frees the text `t` views (line 13), and `t` is used on line 15; print it first: write `println(t)` above line 14
