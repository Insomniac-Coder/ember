# Clause EXC-17.1: "Writing a number or a `bool` (a field of a `Copy` type) moves and frees nothing,
# so it is not a reshape and ends no view; a method that writes `this.health` through a link that
# may lead to its own object is allowed (`[EXC-15]`)."
#$ test: run-pass
#$ rules: EXC-17, EXC-15, BRW-17
#$ stdout: ann
#$ stdout: 9
# A view of `p.name` lives across a number written through a link that may reach `p`; the turret
# aimed at itself writes `health` through its link.

class Player:
    name: String
    hp: int = 5

    fn init(owns name: String):
        this.name = name

class Camera:
    target: weakref Player = None

class Turret:
    health: int = 10
    target: weakref Turret = None

    fn fire():
        if this.target exists as ref mut t:
            t.health -= 1

fn main():
    p = Player(String("ann"))
    cam = Camera()
    cam.target = weakref p
    n = p.name
    if cam.target exists as ref mut t:
        t.hp = 1
    println(n)
    a = Turret()
    a.target = weakref a
    a.fire()
    println(a.health)
