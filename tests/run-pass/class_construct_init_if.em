#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-3, CLS-6
#$ stdout: 42

class Point:
    value: i32

    fn init(choose: bool, value: i32):
        if choose:
            this.value = value
        else:
            this.value = value + 1

fn main():
    point = Point(false, 41)
    println(point.value)
