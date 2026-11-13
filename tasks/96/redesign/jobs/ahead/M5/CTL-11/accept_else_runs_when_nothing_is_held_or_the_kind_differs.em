# Clause CTL-11.3: "`else:` runs when the link or `Option` holds nothing, or when the value is not of
# that kind."
#$ test: run-pass
#$ rules: CTL-11, GRM-44, DSP-4, WK-11
#$ stdout: no value
#$ stdout: gone
#$ stdout: not a circle

interface Shape:
    fn area() -> int

struct Circle:
    r: int

struct Square:
    side: int

extend Circle implements Shape:
    fn area() -> int:
        return 3 * this.r * this.r

extend Square implements Shape:
    fn area() -> int:
        return this.side * this.side

class Player:
    hp: int = 1

fn main():
    o: Option[int] = None
    if o exists as v:
        println(v)
    else:
        println("no value")
    w: weakref Player = None
    if true:
        p = Player()
        w = weakref p
    if w exists as q:
        println(q.hp)
    else:
        println("gone")
    s: owns dyn Shape = owns Square(side=1)
    if s is Circle as c:
        println(c.r)
    else:
        println("not a circle")
