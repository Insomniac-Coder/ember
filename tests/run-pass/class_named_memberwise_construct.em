#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-3, TYP-25
#$ profiles: debug, release, shipping
#$ stdout: 3

class Point:
    x: i32
    y: i32

fn main():
    point = Point(y=2, x=1)
    println(point.x + point.y)
