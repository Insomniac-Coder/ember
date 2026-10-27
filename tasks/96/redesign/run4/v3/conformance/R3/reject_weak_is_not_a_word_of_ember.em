#$ test: compile-fail
#$ rules: R3, R11
#$ help: write `target: weakref Player = None`
## R3 and the removed words of `V3/rules.md` section 3: `Weak[T]` is written `weakref T` (E1010;
## verdict 9; the `Weak` row of `V3/vocabulary.md`).

class Player:
    health: int = 100

class Camera:
    target: Weak[Player]       #$ error[E1010]: `Weak` is not part of Ember

fn main():
    cam = Camera()
