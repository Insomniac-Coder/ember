#$ test: run-pass
#$ rules: R5, R4, R11, R13
#$ stdout: 9
## R5, verdict 18: `changes this` marks only that a method may change its own object; it promises
## nothing about other names. A turret whose `target` is itself reaches itself through `this.target`,
## which is accepted with no check, and `this` is never compiled with noalias (R4), so the write
## through `t` is seen when `this.health` is read again.

class Turret:
    health: int = 10
    target: weakref Turret = None

    fn fire() changes this, Turret -> int:
        if this.target exists as ref mut t:
            t.health -= 1
        return this.health

fn main() changes Turret:
    a = Turret()
    a.target = weakref a
    println(a.fire())
