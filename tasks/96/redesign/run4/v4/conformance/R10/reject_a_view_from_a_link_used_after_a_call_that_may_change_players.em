#$ test: compile-fail
#$ rules: R10, R8, R12, R18
## R10 point 1 (verdict 82): no call in the view's life may end an object of its type, "a function value
## whose type says `changes Player`" included; after such a call the view is not used (E3060). The call
## also changes players, so R8 (g)'s E3021 fits line 17 too; R10 shows the message whose fix compiles,
## E3060 at the later use (verdict 73).

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None
    on_hit: fn() changes Player

    fn show() changes on_hit, Player:
        if this.target exists as p:
            this.on_hit()
            println(p.hp)               #$ error[E3060]: `this.on_hit()` (line 17) may end `Player` objects (its type says `changes Player`), and `p` reads one (`if this.target exists as p:`, line 16) and is used on line 18; use `p` before line 17: move line 18 above it; or open the link again after it: write `if this.target exists as q:` above line 18 and `println(q.hp)` in place of line 18, indented under it

fn main() changes Player:
    player = Player()
    cam = Camera(on_hit=fn(): pass)
    cam.target = weakref player
    cam.show()
