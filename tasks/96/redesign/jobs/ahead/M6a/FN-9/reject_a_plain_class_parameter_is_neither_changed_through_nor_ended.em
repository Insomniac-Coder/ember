# Clause FN-9.1: "`p: Player` reads the object, which neither changes nor ends while `p` is used"
#$ test: compile-fail
#$ rules: FN-9, BRW-21, BRW-15, CLS-1
# `hurt` changes the object through `p` (`E3023`, `[BRW-15]`); `show` ends it, by clearing the list
# that owns it, and then uses `p` (`E3031` at the call, `[BRW-21]`). No text in the spec for these:
# code only. A change by another name is Q5 (accept_a_write_by_another_name_shows_through_p).

class Player:
    hp: int = 5

static mut PLAYERS: Array[Player] = []

fn hurt(p: Player):
    p.hp -= 1    #$ error[E3023]

fn show(p: Player):
    PLAYERS.clear()
    println(p.hp)

fn main():
    PLAYERS.push(owns Player())
    hurt(PLAYERS[0])
    show(PLAYERS[0])    #$ error[E3031]
