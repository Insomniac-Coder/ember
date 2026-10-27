#$ test: run-pass
#$ rules: R12, R10, R13
#$ stdout: 10
#$ stdout: gone
## Verdict 55, R10 point 1 (rules.md lines 508-516): `p`'s last use comes before `reset()`, so it does not
## live across the call; the link opened again after it finds the player ended. The nearest allowed
## program to reject_a_name_opened_from_a_link_used_after_a_call_that_may_end_players (its first way out).

class Player:
    hp: int = 10

class Camera:
    target: weakref Player = None

fn main() changes Player:
    players: Array[Player] = [Player(), Player()]
    cam = Camera()
    cam.target = weakref players[0]
    reset = fn() with owns players changes Player: players.clear()
    if cam.target exists as p:
        println(p.hp)
        reset()
    if cam.target exists as p:
        println(p.hp)
    else:
        println("gone")
