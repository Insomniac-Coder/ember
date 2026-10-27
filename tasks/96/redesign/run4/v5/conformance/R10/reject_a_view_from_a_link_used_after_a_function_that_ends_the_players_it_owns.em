#$ test: compile-fail
#$ rules: BRW-17, LT-3
## design rules: R10, R12, R18
## R10 point 1, R12 (verdicts 55, 82, 86; safety validation (b), hole 3): `reset` owns the list of
## players and clears it, which the compiler works out from its body, so a call of it may end the player
## `p` reads; `p` is not used after the call (E3060, verdict 73: the fix compiles, where R8 (g)'s
## E3021 would not).

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
        reset()
        println(p.hp)                   #$ error[E3060]: `reset()` (line 20) may end `Player` objects (it runs the function stored on line 18, which clears the players it owns), and `p` reads one (`if cam.target exists as p:`, line 19) and is used on line 21; use `p` before line 20: move line 21 above it; or open the link again after it: write `if cam.target exists as q:` above line 21 and `println(q.hp)` in place of line 21, indented under it
