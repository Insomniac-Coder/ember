#$ test: compile-fail
#$ rules: BRW-17, LT-3
## design rules: R10, R13, R18
## R10 point 1 (verdict 82): `remove_all(ref mut players)` is given the list that owns the player `p`
## reads, so it may end it; a name opened from a link is not used after such a call (E3060). R10 keeps
## nothing alive for it while the program runs (rules section 2, "Gone": the count pair).

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn remove_all(ref mut ps: Array[Player]):
    ps.clear()

fn main():
    players = [Player()]
    cam = Camera()
    cam.target = weakref players[0]
    if cam.target exists as p:
        remove_all(ref mut players)
        println(p.hp)                   #$ error[E3060]: `remove_all(ref mut players)` (line 21) may destroy the player `p` reads (`if cam.target exists as p:`, line 20), and `p` is used on line 22; use `p` before line 21: move line 22 above it; or open the link again after it: write `if cam.target exists as q:` above line 22 and `println(q.hp)` in place of line 22, indented under it
