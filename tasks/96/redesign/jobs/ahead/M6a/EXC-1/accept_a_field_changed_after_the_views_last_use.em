# Clause EXC-1.1: "while a read-only view into that field (...) is still used"
#$ test: run-pass
#$ rules: EXC-1, BRW-2
#$ stdout: 5
#$ stdout: ann
#$ stdout: 1 bob
# The same changes, each after the view's last use.

class Player:
    name: String
    hp: int = 5

    fn init(owns name: String):
        this.name = name

class Camera:
    target: weakref Player = None

fn main():
    p = Player(String("ann"))
    cam = Camera()
    cam.target = weakref p
    v = ref p.hp
    println(v)
    n = p.name
    println(n)
    if cam.target exists as ref mut t:
        t.hp = 1
        t.name = String("bob")
    println(p.hp, p.name)
