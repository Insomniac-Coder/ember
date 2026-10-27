#$ test: compile-fail
#$ rules: R3
#$ help: for a new player write `b = copy of a`; for a second owner of this one that changes it write `b = shares mut a`
## R3 and the removed words of `V5/rules.md` section 3: `a.clone()` is written `copy of a` (E1010;
## AN-45's message).

@derive(Clone)
class Player:
    health: int

fn main():
    a = Player(health=100)
    b = a.clone()              #$ error[E1010]: `a.clone()`: a copy is written `copy of a`
    b.health = 5
    println(a.health)
