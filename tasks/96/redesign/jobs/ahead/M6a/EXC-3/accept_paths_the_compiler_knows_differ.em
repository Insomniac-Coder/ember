# Clause EXC-3.1: "unless the compiler knows they differ: classes of which neither is the other nor
# derives from it, a fresh object, or the line inside `if a is not b:`"
#$ test: run-pass
#$ rules: EXC-3, DIA-25
#$ stdout: 8
#$ stdout: 9
#$ stdout: 4
# The message's line (`if t is not a:`), two classes of one family of which neither derives from
# the other (`Turret` and `Rock`), and a fresh object.

open class Entity:
    health: int = 10

    fn init():
        pass

class Rock(Entity):
    fn init():
        super.init()

class Turret(Entity):
    target: weakref Turret = None
    rock: weakref Rock = None

    fn init():
        super.init()

fn hit(ref mut a: Turret):
    if a.target exists as ref mut t:
        if t is not a:
            t.health -= 1
    a.health -= 2
    println(a.health)

fn crush(ref mut a: Turret):
    if a.rock exists as ref mut r:
        r.health -= 1
        println(r.health)
    a.health -= 1

fn main():
    x = Turret()
    x.target = weakref x
    hit(ref mut x)
    stone = Rock()
    x.rock = weakref stone
    crush(ref mut x)
    fresh = Turret()
    fresh.health = 4
    if x.target exists as ref mut t:
        println(fresh.health)
        t.health -= 0
