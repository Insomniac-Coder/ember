#$ test: run-pass
#$ rules: BRW-17, LT-3
#$ stdout: 100
#$ stdout: gone
## design rules: R10, R11, R13
## R10 point 1 (verdict 82): the nearest allowed program to
## `reject_a_view_from_a_link_used_after_a_call_that_may_end_its_player.em`, with both of its message's
## fixes: `p` is used before `remove_all`, and the link is opened again after it, which tests the
## player's header and finds it ended. Nothing keeps the player alive for `p`: the emitted C has no count
## change and no region counter in `p`'s life. Checked under AddressSanitizer: no report.

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
        println(p.hp)                   # 100: `p`'s last use, before the call
        remove_all(ref mut players)     # ends the player
        if cam.target exists as q:
            println(q.hp)
        else:
            println("gone")
