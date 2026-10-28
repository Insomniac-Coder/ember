#$ test: run-pass
#$ rules: BRW-16, FN-4
#$ stdout: bo 11
## design rules: R8, R5
## R8 (f), R5: the compiler works out from `heal`'s body that it changes `hp` only (verdict 86), so it
## may run while `n` is the only name for `p.name`: the two never name one place. Completeness check,
## G22.

class Player:
    name: String
    hp: int = 10

    fn heal():
        this.hp += 1

fn main():
    p = Player(name=String("ann"))
    n = ref mut p.name
    p.heal()
    n = String("bo")
    println(p.name, p.hp)
