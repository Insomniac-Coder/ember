#$ test: run-pass
#$ rules: R10, R11
#$ stdout: 100
#$ stdout: gone
## R10 point 3: "Otherwise a count pair around the view's life or around the call": `drop_all` may end
## the player `p` views (it ends a `Player` without changing one), and no loop is in `p`'s life, so
## `p` holds a count across the call; the player ends when `p`'s life ends. Section 2's R10 row, "a
## count pair". Checked under AddressSanitizer: no report; in the emitted C, one +1 before the call and
## one -1 after `p`'s last use. See the matrix row R10-3 for the question this test leaves open (R8 (b)).

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn drop_all(ref mut ps: Array[Player]):
    ps.clear()

fn main():
    players: Array[Player] = []
    cam = Camera()
    first = Player()
    cam.target = weakref first
    players.push(first)
    if cam.target exists as p:
        drop_all(ref mut players)       # ends the player's only owner while `p` is used
        println(p.hp)                   # 100: `p` keeps the player alive
    if cam.target exists as q:
        println("alive")
    else:
        println("gone")                 # the player ended with `p`'s life
