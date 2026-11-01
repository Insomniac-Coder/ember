#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-3, TYP-25, EXP-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ 32

class Point:
    x: i32
    y: i32

    fn init(x: i32, y: i32):
        this.x = x
        this.y = y

fn bump(ref mut value: i32) -> i32:
    value = value + 1
    return value

fn main():
    point = Point(y=23, x=19)
    println(point.x + point.y)
    state: i32 = 1
    point = Point(y=bump(ref mut state), x=bump(ref mut state))
    println(point.x * 10 + point.y)
