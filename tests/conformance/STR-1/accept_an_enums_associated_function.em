#$ test: run-pass
#$ rules: STR-1, ENM-1
#$ stdout: Shape.Circle(r=1) Shape.Dot
# D-304 — an enum's associated function (a `fn` with no receiver) is called
# on the enum, as a struct's is: `Shape.unit()`. A name that is no variant
# was looked up only as one, so the call was E1010 "has no variant".

enum Shape:
    Circle(r: int)
    Dot

    fn unit() -> Shape:
        return Shape.Circle(1)

    fn point() -> Self:
        return Shape.Dot

fn main():
    println(Shape.unit(), Shape.point())
