#$ test: run-pass
#$ rules: OWN-1, BRW-12
#$ stdout: 9
## design rules: R1, R2
## R1, R2: the nearest program that is allowed: `cur = ref mut p` names the caller's player through
## `p` and may change it.

class Player:
    hp: int = 10

fn hurt(ref mut p: Player):
    cur = ref mut p
    cur.hp -= 1

fn main():
    player = Player()
    hurt(ref mut player)
    println(player.hp)
