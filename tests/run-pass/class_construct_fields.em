#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-3, OWN-2
#$ assert-c: contains(ember_obj_new(&em_ti_Point))
#$ stdout: 42

class Point:
    x: i32
    y: i32

    fn sum(self) -> i32:
        return self.x + self.y

fn main():
    point = Point(19, 23)
    println(point.sum())
