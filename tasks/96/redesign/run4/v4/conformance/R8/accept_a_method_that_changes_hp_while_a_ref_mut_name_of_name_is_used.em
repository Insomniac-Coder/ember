#$ test: run-pass
#$ rules: R8, R5
#$ stdout: bo 11
## R8 (f), R5: `heal`'s first line says it changes `hp` only (row 3's field list), so it may run while `n`
## is the only name for `p.name`: the two never name one place. Completeness check, G22.

class Player:
    name: String
    hp: int = 10

    fn heal() changes hp:
        this.hp += 1

fn main():
    p = Player(name=String("ann"))
    n = ref mut p.name
    p.heal()
    n = String("bo")
    println(p.name, p.hp)
