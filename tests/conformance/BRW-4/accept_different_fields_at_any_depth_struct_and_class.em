# Clause BRW-4.1: "`ref mut a.x` and `ref mut a.y` may be in use together when `x` and `y` are
# different fields, through any depth of field path, for a struct value's fields and a class
# object's alike (`[EXC-19]`)."
#$ test: run-pass
#$ rules: BRW-4, EXC-19, BCK-3
#$ stdout: 30
#$ stdout: 7 8 0
#$ stdout: 11 12

struct P:
    x: int
    y: int

struct Inner:
    a: int
    b: int

struct Outer:
    inner: Inner
    tag: int

class Ship:
    pos: P
    fuel: int = 0

fn main():
    p = P(x=1, y=2)
    a = ref mut p.x
    b = ref mut p.y
    a += 9
    b += 18
    println(p.x + p.y)
    o = Outer(inner=Inner(a=1, b=2), tag=3)
    ia = ref mut o.inner.a
    ib = ref mut o.inner.b
    t = ref mut o.tag
    ia += 6
    ib += 6
    t -= 3
    println(o.inner.a, o.inner.b, o.tag)
    s = Ship(pos=P(x=0, y=0))
    sx = ref mut s.pos.x
    f = ref mut s.fuel
    sx += 11
    f += 12
    println(s.pos.x, s.fuel)
