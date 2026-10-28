#$ test: compile-fail
#$ rules: BRW-21
#$ help: in `heal`, read it through `p`: `println(p.hp)`
## design rules: R4
## R4 (E3031, section 3's fourth message): a `ref mut` parameter is the only way into its place while
## it is used; `heal` reads `PARTY[0]` by its own name while `p` is `PARTY[0]` (step 4's re-check:
## no test). Verdict 88's one-variable copy is for a whole variable, not an element.

class Player:
    hp: int = 10

static mut PARTY: Array[Player] = []

fn heal(ref mut p: Player):
    println(PARTY[0].hp)
    p.hp += 1

fn main():
    PARTY.push(Player())
    heal(ref mut PARTY[0])    #$ error[E3031]: `heal` reads `PARTY[0]` by its own name (its line 2) while `p` is still used (its line 3), and `p` is `PARTY[0]`; in `heal`, read it through `p`: `println(p.hp)`
