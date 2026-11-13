# Clause CTL-11.4: "`exists as copy of v` is a copy of the program's own" -- and `ref mut` in the
# header names the place so that it may change it (`[GRM-44]`, `[DSP-4]`).
#$ test: run-pass
#$ rules: CTL-11, GRM-44, DSP-4, BRW-14
#$ stdout: 2 1
#$ stdout: 5
#$ stdout: 4

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
    o: Option[int] = Some(1)
    if o exists as copy of v:
        v += 1
        println(v, o.unwrap())
    e: Option[Enemy] = Some(Enemy(hp=1))
    if e exists as ref mut en:
        en.hp = 5
    if e exists as en2:
        println(en2.hp)
    s: owns dyn Shape = owns Circle(r=1)
    if s is Circle as ref mut c:
        c.r = 4
    if s is Circle as c2:
        println(c2.r)
