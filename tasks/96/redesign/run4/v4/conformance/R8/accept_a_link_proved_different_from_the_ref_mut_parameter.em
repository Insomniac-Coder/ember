#$ test: run-pass
#$ rules: R8, R13, R5
#$ stdout: 9 9
## R8 (f): inside `if t is not a:` the compiler knows `t` names another turret, so it may change it
## while `a` is still used.
## `hit` changes a turret through the link `a.target` (`t.health -= 1`), so it says `changes Turret`,
## and so does `main`, which calls it (R5, verdict 63).

class Turret:
    health: int = 10
    target: weakref Turret = None

fn hit(ref mut a: Turret) changes Turret:
    if a.target exists as ref mut t:
        if t is not a:
            t.health -= 1
    a.health -= 1

fn main() changes Turret:
    x = Turret()
    y = Turret()
    x.target = weakref y
    hit(ref mut x)
    println(x.health, y.health)
