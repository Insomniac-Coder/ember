#$ test: run-pass
#$ rules: R8, R13
#$ stdout: 9 9
## R8 (f): inside `if t is not a:` the compiler knows `t` names another turret, so it may change it
## while `a` is still used.

class Turret:
    health: int = 10
    target: weakref Turret = None

fn hit(ref mut a: Turret):
    if a.target exists as ref mut t:
        if t is not a:
            t.health -= 1
    a.health -= 1

fn main():
    x = Turret()
    y = Turret()
    x.target = weakref y
    hit(ref mut x)
    println(x.health, y.health)
