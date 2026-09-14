#$ test: compile-fail
#$ rules: CLS-1, CLS-2
#$ error[E2100]: field `value` is not definitely initialized by class `init`

class Point:
    value: i32

    fn init(mut self, choose: bool, value: i32):
        if choose:
            self.value = value

fn main():
    point = Point(false, 41)
    println(point.value)
