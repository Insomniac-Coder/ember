#$ test: compile-fail
#$ rules: R10, R12, R18
## R10 point 1, R12 (verdicts 55, 82; safety validation (b), hole 3): `reset` owns the list of players
## and clears it, so its type says `changes Player`, and a call of it may end the player `p` reads;
## `p` is not used after the call (E3060, verdict 73: the fix compiles, where R8 (g)'s E3021 would not).

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn main() changes Player:
    players = [Player()]
    cam = Camera()
    cam.target = weakref players[0]
    reset = fn() with owns players changes Player: players.clear()
    if cam.target exists as p:
        reset()
        println(p.hp)                   #$ error[E3060]: `reset()` (line 19) may end `Player` objects (its type says `changes Player`), and `p` reads one (`if cam.target exists as p:`, line 18) and is used on line 20; use `p` before line 19: move line 20 above it; or open the link again after it: write `if cam.target exists as q:` above line 20 and `println(q.hp)` in place of line 20, indented under it
