# Clause BRW-1.8: "`this` is not such a name (`[EXC-15]`)."
#$ test: run-pass
#$ rules: BRW-1, EXC-15, BRW-22, WK-11, GRM-44
#$ stdout: 2
# While `fire` uses `this`, the same object is changed through a link (a turret aimed at itself,
# verdict 18): accepted with no check, and `this` sees the change.

class Turret:
    health: int = 3
    target: weakref Turret = None
    fn fire():
        if this.target exists as ref mut t:
            t.health -= 1
        println(this.health)

fn main():
    turret = Turret()
    turret.target = weakref turret
    turret.fire()
