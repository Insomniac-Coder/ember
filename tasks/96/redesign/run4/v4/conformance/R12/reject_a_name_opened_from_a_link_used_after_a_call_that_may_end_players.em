#$ test: compile-fail
#$ rules: R12, R10, R13
#$ help: use `p` before line 22
## Verdict 55, R10 point 1 (rules.md lines 508-516, 520-526): `reset`'s type says `changes Player`, so a
## call of it may end the player `p` reads; a name opened from a link is not used after such a call
## (E3060), and nothing runs for it while the program runs. rules.md section 3's line 1017 third message,
## in this program's lines; its second way out opens the link under a new name, since `exists as p` again
## inside this block would name a name in use (E2106, R13).

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
        reset()
        println(p.hp)    #$ error[E3060]: `reset()` (line 22) may end `Player` objects (its type says `changes Player`), and `p` reads one (`if cam.target exists as p:`, line 21) and is used on line 23; use `p` before line 22, or open the link again after it: write `if cam.target exists as q:` above line 23, indent line 23 under it and use `q` there: `println(q.hp)`
