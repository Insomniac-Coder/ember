# Clause FN-9.1: "`p: Player` reads the object, which neither changes nor ends while `p` is used"
#$ test: run-pass
#$ rules: FN-9, FN-1, BRW-21
#$ stdout: 4
# Q5's reading (B): `[BRW-21]` holds a plain parameter's callee only to not ending or moving what it
# views, and `[FN-1]` lets a number another name writes show through (M5's Q9, the lead's Q1);
# "neither changes" is read as "through `p`". Under reading (A) this is refused.

class Player:
    hp: int = 5

static mut PLAYERS: Array[Player] = []

fn weaken_and_show(p: Player):
    PLAYERS[0].hp -= 1
    println(p.hp)

fn main():
    PLAYERS.push(owns Player())
    weaken_and_show(PLAYERS[0])
