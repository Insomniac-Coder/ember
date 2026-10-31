#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-3, CLS-6
#$ assert-c: contains(ember_obj_new(&em_ti_5Point))
#$ assert-c: contains(em_5Point4init)
#$ stdout: 42

class Point:
    x: i32
    y: i32

    fn init(x: i32, y: i32):
        this.x = x
        this.y = y

fn main():
    point = Point(19, 23)
    println(point.x + point.y)
