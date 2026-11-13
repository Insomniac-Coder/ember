# Clause CTL-11.4: "The name keeps its place for the whole block (`[BRW-14]`): a plain one only
# reads, and `exists as copy of v` is a copy of the program's own."
#$ test: compile-fail
#$ rules: CTL-11, BRW-14, BRW-15
# The spec gives no message for these: codes only.

interface Shape:
    fn area() -> int

struct Circle:
    r: int

extend Circle implements Shape:
    fn area() -> int:
        return 3 * this.r * this.r

struct Enemy:
    hp: int

fn main():
    o: Option[Enemy] = Some(Enemy(hp=1))
    if o exists as e:
        e = Enemy(hp=2)    #$ error[E3023]
    if o exists as f:
        f.hp = 3    #$ error[E3023]
    s: owns dyn Shape = owns Circle(r=1)
    if s is Circle as c:
        c.r = 2    #$ error[E3023]
