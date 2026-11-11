#$ test: run-pass
#$ rules: WK-12, WK-11, GRM-44
#$ profiles: debug, development, release
#$ stdout: 11
#$ ended
# `[WK-12]` — a `weakref` link is read by opening it with `exists as`: the block runs, with a view
# of the object, while the target lives; `else:` runs once it has ended. `[WK-11]` — a link does not
# keep its target alive.

class Player:
    hp: int

fn link_to_a_player_that_ends() -> weakref Player:
    p = Player(hp=5)
    w = weakref p
    return w

fn main():
    p = Player(hp=10)
    w = weakref p
    if w exists as q:
        println(q.hp + 1)
    else:
        println(0)
    gone = link_to_a_player_that_ends()
    if gone exists as q:
        println(q.hp)
    else:
        println("ended")
