#$ test: compile-fail
#$ rules: CLS-1, CLS-2, CLS-3, TYP-25
#$ profiles: debug, development, release
#$ error[E2021]: `Point(missing=1, y=2)` names `missing`, and `Point` has no parameter of that name: `fn init(x: i32, y: i32)`
#$ error[E2021]: `Point(x=1, x=2)` gives `x` twice: `fn init(x: i32, y: i32)`
#$ error[E2021]: `Point(y=2, 1)` gives `1` after the named `y=2`, and positional arguments come first; write `Point(y=2, x=1)`

class Point:
    x: i32
    y: i32

    fn init(x: i32, y: i32):
        this.x = x
        this.y = y

fn main():
    Point(missing=1, y=2)
    Point(x=1, x=2)
    Point(y=2, 1)
