#$ test: compile-fail
#$ rules: R10, R13, R18
## R10 point 1 (verdict 82): a loop of calls that may end a `Player` is in `p`'s life, so `p` is not
## used after it (E3060). The v3 rules kept the player alive with a deferred-drop region around `p`'s
## life; R10 no longer does (rules section 2, "Gone").

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn drop_last(ref mut ps: Array[Player]):
    if ps.len() > 0:
        ps.pop()

fn main():
    players: Array[Player] = []
    cam = Camera()
    first = Player()
    cam.target = weakref first
    players.push(first)
    players.push(Player(hp=5))
    if cam.target exists as p:
        for _ in 0..2:
            drop_last(ref mut players)  # the second call ends the player `p` reads
        println(p.hp)                   #$ error[E3060]: `drop_last(ref mut players)` (line 26) may destroy the player `p` reads (`if cam.target exists as p:`, line 24), and `p` is used on line 27; use `p` before the loop: move line 27 above line 25; or open the link again after the loop: write `if cam.target exists as q:` above line 27 and `println(q.hp)` in place of line 27, indented under it
