#$ test: run-pass
#$ rules: OWN-1
#$ stdout: 30
#$ 10
## design rules: R1
## R1: where `a` is itself a view (a plain parameter, a local holding a view), `b = a` makes `b` a
## read-only view of what `a` views, and `ref` may say so (`other = ref p`); a view hands nothing over,
## so `p`, `cur` and `other` all read the caller's player, and the caller keeps it.

class Player:
    hp: int = 10

fn total(p: Player) -> int:
    cur = p
    other = ref p
    return cur.hp + other.hp + p.hp

fn main():
    player = Player()
    println(total(player))
    println(player.hp)
