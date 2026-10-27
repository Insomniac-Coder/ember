#$ test: run-pass
#$ rules: BRW-17, LT-3
#$ stdout: gone
## design rules: R12, R5, R13
## Verdicts 55 and 86, R12 (rules.md lines 681-685): "`reset = fn() owns players: players.clear()` may end
## players", worked out from its body with no word on it or on `main`, which runs it; the link opened after
## the call finds its player ended. A name opened before the call and used after it is refused
## (reject_a_name_opened_from_a_link_used_after_a_call_that_may_end_players).

class Player:
    hp: int = 10

fn main():
    players: Array[Player] = [Player(), Player()]
    w = weakref players[0]
    reset = fn() owns players: players.clear()
    reset()
    if w exists as p:
        println(p.hp)
    else:
        println("gone")
