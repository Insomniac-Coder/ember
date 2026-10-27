#$ test: run-pass
#$ rules: R12, R10, R13
#$ stdout: 10
#$ stdout: gone
## Verdicts 55 and 82, R12 (rules.md lines 684-685), R10 point 1 (lines 564-574): `p`'s last use comes
## before `reset()`, so it does not live across the call; the link opened again after it finds the player
## ended. The nearest allowed program to
## reject_a_name_opened_from_a_link_used_after_a_call_that_may_end_players (its first way out).

class Player:
    hp: int = 10

class Camera:
    target: weakref Player = None

fn main():
    players: Array[Player] = [Player(), Player()]
    cam = Camera()
    cam.target = weakref players[0]
    reset = fn() owns players: players.clear()
    if cam.target exists as p:
        println(p.hp)
        reset()
    if cam.target exists as p:
        println(p.hp)
    else:
        println("gone")
