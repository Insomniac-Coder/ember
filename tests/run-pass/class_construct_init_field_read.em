#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-3
#$ stdout: 42
#$ stdout: 42

class Point:
    value: i32

    fn init(value: i32):
        this.value = value
        println(this.value)

fn main():
    point = Point(42)
    println(point.value)
