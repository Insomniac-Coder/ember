#$ test: compile-fail
#$ rules: BRW-1
#$ help: if t is not a:
## design rules: R8, R13, R5
## R8 (f): another name that may name a `ref mut` name's place, a link of the same type included, is
## refused unless the compiler proves the two differ. Rules section 3's E3022 message for the turret, in
## this program's lines. `a` is a `ref mut` parameter, which is alone while it is used; a method's
## `this` is not (verdict 18: the turret aimed at itself through `this` is allowed, R5).

class Turret:
    health: int = 10
    target: weakref Turret = None

fn hit(ref mut a: Turret):
    if a.target exists as ref mut t:
        t.health -= 1    #$ error[E3022]: `a` is the only name for its turret while `hit` runs, and `t`, opened from `a.target`, may be that turret; write `if t is not a:` above line 15 and indent line 15 under it
    a.health -= 1

fn main():
    x = Turret()
    x.target = weakref x
    hit(ref mut x)
