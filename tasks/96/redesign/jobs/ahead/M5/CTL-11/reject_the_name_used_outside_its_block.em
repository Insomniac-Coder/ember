# Clause CTL-11.2: "and the name they declare is in scope in that block only, never in an `elif` or
# `else`"
#$ test: compile-fail
#$ rules: CTL-11, GRM-44, DSP-4
# The spec gives no message for these uses; a name not in scope is `E1010` (§XVII's registry). Codes
# only.

interface Shape:
    fn area() -> int

struct Circle:
    r: int

extend Circle implements Shape:
    fn area() -> int:
        return 3 * this.r * this.r

fn main():
    o: Option[int] = None
    if o exists as v:
        println(v)
    else:
        println(v)    #$ error[E1010]
    println(v)    #$ error[E1010]
    s: owns dyn Shape = owns Circle(r=1)
    if s is Circle as c:
        println(c.r)
    elif s.area() > 0:
        println(c.r)    #$ error[E1010]
