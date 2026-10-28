#$ test: compile-fail
#$ rules: MOD-3
#$ help: write `target: weakref Player = None`
## design rules: R3, R11
## R3 and the removed words of `V4/rules.md` section 3: `Weak[T]` is written `weakref T` (E1010;
## verdict 9; the `Weak` row of `V4/vocabulary.md`).

class Player:
    health: int = 100

class Camera:
    target: Weak[Player]       #$ error[E1010]: `Weak` is not part of Ember

fn main():
    println(0)
