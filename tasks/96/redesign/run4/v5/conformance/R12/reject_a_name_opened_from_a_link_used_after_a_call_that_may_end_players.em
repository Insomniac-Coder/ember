#$ test: compile-fail
#$ rules: BRW-17, LT-3
#$ help: use `p` before line 24
## design rules: R12, R10, R13
## Verdicts 55, 82 and 86, R12 (rules.md lines 684-685: "`reset = fn() owns players: players.clear()` may
## end players, so a view of a player opened from a link is not used after `reset()` (R10, E3060)"), R10
## point 1 (lines 564-574): the compiler works out from the function stored in `reset` that the call may end
## a `Player`, and nothing runs for it while the program runs. rules.md section 3's line 1069 third
## message, in this program's lines; its second way out opens the link under a new name (F-01), since
## `exists as p` again inside this block would name a name in use (E2106, R13). The nearest allowed
## program is accept_a_name_opened_from_a_link_used_before_the_call_that_may_end_players.

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
        reset()
        println(p.hp)    #$ error[E3060]: `reset()` (line 24) may end `Player` objects (it runs the function stored on line 22, which clears the players it owns), and `p` reads one (`if cam.target exists as p:`, line 23) and is used on line 25; use `p` before line 24, or open the link again after it: write `if cam.target exists as q:` above line 25, indent line 25 under it and use `q` there
