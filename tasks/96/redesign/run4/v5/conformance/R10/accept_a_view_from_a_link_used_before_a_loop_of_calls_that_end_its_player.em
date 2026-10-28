#$ test: run-pass
#$ rules: BRW-17, LT-3
#$ stdout: 100
#$ stdout: gone
## design rules: R10, R11, R13
## R10 point 1 (verdict 82): the nearest allowed program to
## `reject_a_view_from_a_link_used_after_a_loop_of_calls_that_may_end_players.em`: `p` is used before the
## loop, and the link is opened again after it, which finds the player ended. No region and no count:
## the emitted C has neither around the loop. Checked under AddressSanitizer: no report, no leak.

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
        println(p.hp)                   # 100: `p`'s last use, before the loop
        for _ in 0..2:
            drop_last(ref mut players)  # the second call ends the player
        if cam.target exists as q:
            println(q.hp)
        else:
            println("gone")
