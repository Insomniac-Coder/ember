#$ test: run-pass
#$ rules: R12, R5, R13
#$ stdout: gone
## Verdict 55, R12 (rules.md lines 611-617): `reset` names `Player`, the type of the class objects it may
## end, and `main`, which runs it, names it too (R5); the link opened after the call finds its player
## ended. The nearest allowed program to
## reject_a_function_that_ends_the_players_it_owns_without_naming_player.

class Player:
    hp: int = 10

fn main() changes Player:
    players: Array[Player] = [Player(), Player()]
    w = weakref players[0]
    reset = fn() with owns players changes Player: players.clear()
    reset()
    if w exists as p:
        println(p.hp)
    else:
        println("gone")
