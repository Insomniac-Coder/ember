#$ test: run-pass
#$ rules: R10, R11
#$ stdout: 100
#$ stdout: gone
## R10 point 2: "A deferred-drop region, where the view's life contains a loop that makes a call that
## may end such an owner and the class has no `drop` method of its own. The region ... [is placed]
## around the view's whole life when the view is used after the loop": `p` is used after the loop,
## so the region covers `p`'s life; the player `drop_last` ends is set aside and its memory freed when
## the region ends. Section 2's R10 row, "a deferred-drop region": checked in the emitted C, one add
## and one subtract of the per-thread counter around `p`'s life and nothing in the loop; checked under
## AddressSanitizer, no report.

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
        for i in 0..2:
            drop_last(ref mut players)  # may end a `Player`: the second call ends the one `p` views
        println(p.hp)                   # 100: its memory is kept until the region ends
    if cam.target exists as q:
        println("alive")
    else:
        println("gone")
