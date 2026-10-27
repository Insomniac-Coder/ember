#$ test: run-pass
#$ rules: R8, R13
#$ stdout:
#$ 10 10
#$ 10 20
## R8 (g): a loop name and a name opened from a link inside the walk may be one player, and both only
## read, so nothing is refused; nothing in the loop can end a player, so nothing runs for `q` (R10
## point 1). The nearest program to `reject_a_link_opened_in_a_ref_mut_walk_reads_the_loop_names_player.em`
## that is allowed. Completeness check, G25.

class Player:
    hp: int = 10

fn main():
    players: Array[Player] = []
    first = Player(hp=10)
    w = weakref first
    players.push(first)
    players.push(Player(hp=20))
    for p in players:
        if w exists as q:
            println(q.hp, p.hp)
