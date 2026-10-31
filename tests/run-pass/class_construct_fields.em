#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-3, OWN-2
#$ assert-c: contains(&em_ti_5Point))
#$ stdout: 42

class Point:
    x: i32
    y: i32

    fn sum() -> i32:
        return this.x + this.y

fn main():
    point = Point(19, 23)
    println(point.sum())
