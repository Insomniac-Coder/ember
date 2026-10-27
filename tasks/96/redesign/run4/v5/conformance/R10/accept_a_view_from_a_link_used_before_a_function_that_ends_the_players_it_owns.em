#$ test: run-pass
#$ rules: R10, R12, R13
#$ stdout: 100
#$ stdout: gone
## R10 point 1, R12 (verdicts 55, 82): the nearest allowed program to
## `reject_a_view_from_a_link_used_after_a_function_that_ends_the_players_it_owns.em`: `p` is used before
## `reset()`, and the link is opened again after it, which finds the player ended. Checked under
## AddressSanitizer: no report.

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn main():
    players = [Player()]
    cam = Camera()
    cam.target = weakref players[0]
    reset = fn() owns players: players.clear()
    if cam.target exists as p:
        println(p.hp)                   # 100: `p`'s last use, before the call
        reset()                         # ends the player, which the function's own state owned
        if cam.target exists as q:
            println(q.hp)
        else:
            println("gone")
