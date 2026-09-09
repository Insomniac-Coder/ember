#$ test: run-pass
#$ rules: BRW-4
# "`ref mut a.x` and `ref mut a.y` may be live simultaneously if `x` and `y`
# are distinct fields of a struct/tuple."

struct P:
    pub x: i32
    pub y: i32

fn main():
    p = P(1, 2)
    a: ref mut i32 = ref mut p.x
    b: ref mut i32 = ref mut p.y
    a = 10
    b = 20
    println(p.x + p.y)
#$ stdout: 30
