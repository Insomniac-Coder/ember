#$ test: compile-fail
#$ rules: CLS-1, CLS-2
#$ error[E1010]: class `init` cannot overwrite conditionally initialized field `value` in this phase

class Point:
    value: i32

    fn init(mut self, choose: bool, value: i32):
        if choose:
            self.value = value
        self.value = value + 1

fn main():
    point = Point(true, 41)
    println(point.value)
