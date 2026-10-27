#$ test: compile-fail
#$ rules: R10, R8, R12, R18
#$ help: if this.target exists as q:
## R10 point 1 (verdicts 82, 86): no call in the view's life may end an object of its type, a call
## through a function value included, by every function the program can put there: `on_hit` holds
## `fn() owns players: players.clear()`, which ends the players its state owns, so `p` is not used after
## `this.on_hit()` (E3060). The call also changes players, so R8 (g)'s E3021 fits line 19 too; R10
## shows the message whose fix compiles, E3060 at the later use (verdict 73).

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None
    on_hit: fn()

    fn show():
        if this.target exists as p:
            this.on_hit()
            println(p.hp)               #$ error[E3060]: `this.on_hit()` (line 19) may end `Player` objects (it may run the function stored on line 25, which clears the players it owns), and `p` reads one (`if this.target exists as p:`, line 18) and is used on line 20; use `p` before line 19: move line 20 above it; or open the link again after it: write `if this.target exists as q:` above line 20, indent line 20 under it and use `q` there

fn main():
    players = [Player()]
    w = weakref players[0]
    cam = Camera(on_hit=fn() owns players: players.clear())
    cam.target = w
    cam.show()
