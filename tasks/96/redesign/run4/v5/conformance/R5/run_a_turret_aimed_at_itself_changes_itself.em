#$ test: run-pass
#$ rules: R5, R4, R11, R13
#$ stdout: 9
## R5, verdict 18: that a method may change its own object (worked out from its body, verdict 86)
## promises nothing about other names. A turret whose `target` is itself reaches itself through `this.target`,
## which is accepted with no check, and `this` is never compiled with noalias (R4), so the write
## through `t` is seen when `this.health` is read again. `fire` uses `this` after a change through
## `t`, but that change writes a number and can end or move nothing, so `a.fire()` is not the call
## verdict 54 refuses (E3031 is for a change that may end what `this` is part of, `ENEMIES.clear()`).

class Turret:
    health: int = 10
    target: weakref Turret = None

    fn fire() -> int:
        if this.target exists as ref mut t:
            t.health -= 1
        return this.health

fn main():
    a = Turret()
    a.target = weakref a
    println(a.fire())
