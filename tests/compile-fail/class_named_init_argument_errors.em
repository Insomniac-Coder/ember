#$ test: compile-fail
#$ rules: CLS-1, CLS-2, CLS-3, TYP-25
#$ profiles: debug, release, shipping
#$ error[E2020]: has no parameter `missing`
#$ error[E1030]: parameter `x` is given twice
#$ error[E2020]: positional arguments must come before named ones

class Point:
    x: i32
    y: i32

    fn init(mut self, x: i32, y: i32):
        self.x = x
        self.y = y

fn main():
    Point(missing=1, y=2)
    Point(x=1, x=2)
    Point(y=2, 1)
