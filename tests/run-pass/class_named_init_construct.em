#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-3, TYP-25, EXP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ 32

class Point:
    x: i32
    y: i32

    fn init(mut self, x: i32, y: i32):
        self.x = x
        self.y = y

fn bump(mut value: i32) -> i32:
    value = value + 1
    return value

fn main():
    point = Point(y=23, x=19)
    println(point.x + point.y)
    state = 1
    point = Point(y=bump(state), x=bump(state))
    println(point.x * 10 + point.y)
