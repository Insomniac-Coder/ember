#$ test: run-pass
#$ rules: R4
#$ stdout: 10
#$ stdout: 11
## R4: the nearest allowed program to `reject_a_ref_mut_argument_the_callee_also_reads_by_its_own_name`:
## `heal` reads the player through `p`, the only way into it while it is used.

class Player:
    hp: int = 10

static mut PARTY: Array[Player] = []

fn heal(ref mut p: Player):
    println(p.hp)
    p.hp += 1

fn main():
    PARTY.push(Player())
    heal(ref mut PARTY[0])
    println(PARTY[0].hp)
