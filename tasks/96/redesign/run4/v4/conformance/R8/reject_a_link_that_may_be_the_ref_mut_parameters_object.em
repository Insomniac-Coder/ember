#$ test: compile-fail
#$ rules: R8, R13, R5
#$ help: if t is not a:
## R8 (f): another name that may name a `ref mut` name's place, a link of the same type included, is
## refused unless the compiler proves the two differ. The E3022 message of `V4/rules.md` section 3.
## `hit` changes a turret through the link `a.target` (`t.health -= 1`), so it says `changes Turret`,
## and so does `main`, which calls it (R5, verdict 63).

class Turret:
    health: int = 10
    target: weakref Turret = None

fn hit(ref mut a: Turret) changes Turret:
    if a.target exists as ref mut t:
        t.health -= 1    #$ error[E3022]: `a` is the only name for its turret while `hit` runs, and `t`, opened from `a.target`, may be that turret; test `if t is not a:` first
    a.health -= 1

fn main() changes Turret:
    x = Turret()
    x.target = weakref x
    hit(ref mut x)
