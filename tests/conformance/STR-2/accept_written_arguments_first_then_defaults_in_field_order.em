#$ test: run-pass
#$ rules: STR-2, EXP-1, FN-5
#$ stdout: arg d
#$ arg b
#$ default a
#$ default c
#$ 1 2 3 4
#$ arg y
#$ arg x
#$ default a
#$ default b
#$ body
#$ 1 2 3 4
## `[STR-2]` - at a construction that omits fields, the written arguments are evaluated first, left to
## right, then the omitted fields' defaults in field order; where the type declares an `init`, every
## field's default runs before its body, after the arguments.

fn trace(tag: String, v: int) -> int:
    println(tag)
    return v

struct Ord4:
    a: int = trace("default a", 1)
    b: int
    c: int = trace("default c", 3)
    d: int

struct W:
    a: int = trace("default a", 1)
    b: int = trace("default b", 2)
    x: int
    y: int

    fn init(y: int, x: int):
        println("body")
        this.x = x
        this.y = y

fn main():
    o = Ord4(d=trace("arg d", 4), b=trace("arg b", 2))
    println(o.a, o.b, o.c, o.d)
    w = W(trace("arg y", 4), trace("arg x", 3))
    println(w.a, w.b, w.x, w.y)
