#$ test: run-pass
#$ rules: R3
#$ stdout: 100 5
## R3 and `V4/rules.md` section 2 ("a copy, an allocation, where a line says `copy of`"): `copy of a`
## makes a new player by its type's `Clone`; nothing done to one is seen through the other (AN-45,
## where today's `.clone()` of a class shares the object and prints 5). Verified by its output and a
## clean AddressSanitizer run (two blocks, each freed once).

@derive(Clone)
class Player:
    health: int

fn main():
    a = Player(health=100)
    b = copy of a
    b.health = 5
    println(a.health, b.health)
