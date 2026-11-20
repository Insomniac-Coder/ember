# Clause EXC-3.1: "Two paths may reach the same object when they lead to objects of one class
# family (the classes that share one root base class) and at least one goes through a link or a
# kept view, unless the compiler knows they differ"; "A possible overlap the compiler cannot rule
# out is refused (`E3022` for a change, `E3021` for a read), with the test that proves them apart
# as the line to write"
#$ test: compile-fail
#$ rules: EXC-3, BRW-1, BRW-22
# The spec's message, with this file's line numbers, for a change; a read through a link of a base
# class's type (one family) has no text in the spec: code only.

open class Entity:
    health: int = 10

    fn init():
        pass

class Turret(Entity):
    target: weakref Turret = None
    aim: weakref Entity = None

    fn init():
        super.init()

fn hit(ref mut a: Turret):
    if a.target exists as ref mut t:
        t.health -= 1    #$ error[E3022]: `a` is the only name for its turret while `hit` runs, and `t`, opened from `a.target`, may be that turret; write `if t is not a:` above line 26 and indent line 26 under it
    a.health -= 2

fn peek(ref mut a: Turret):
    if a.aim exists as e:
        println(e.health)    #$ error[E3021]
    a.health -= 1

fn main():
    x = Turret()
    x.target = weakref x
    hit(ref mut x)
    peek(ref mut x)
