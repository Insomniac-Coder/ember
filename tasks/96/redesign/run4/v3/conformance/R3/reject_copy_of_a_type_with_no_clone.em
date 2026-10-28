#$ test: compile-fail
#$ rules: R3
#$ help: give it one: `@derive(Clone)` above `class Player`
## R3: `copy of` is refused before a type with no `Clone` (E2116); the message says how to give it one
## (`V3/rules.md` section 3; AN-45). The nearest program that is allowed is
## `run_copy_of_makes_an_independent_object`.

class Player:
    health: int

fn main():
    a = Player(health=100)
    b = copy of a              #$ error[E2116]: `Player` has no `Clone`, so `copy of` cannot make one
    b.health = 5
    println(a.health)
