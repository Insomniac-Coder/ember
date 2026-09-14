#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-3
#$ stdout: 42
#$ stdout: 42

class Point:
    value: i32

    fn init(mut self, value: i32):
        self.value = value
        println(self.value)

fn main():
    point = Point(42)
    println(point.value)
