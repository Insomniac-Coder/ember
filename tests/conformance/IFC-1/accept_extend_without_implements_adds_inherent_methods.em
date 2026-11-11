#$ test: run-pass
#$ rules: IFC-1
#$ stdout: 7
# `[IFC-1]` — `extend T:` without `implements` adds inherent methods to `T`.

struct Point:
    x: int
    y: int

extend Point:
    fn sum() -> int:
        return this.x + this.y

fn main():
    println(Point(x=3, y=4).sum())
