#$ test: compile-fail
#$ rules: R10, R8, R12
## R10: "Across a call that may change one (its first line or its type says `changes T`), a read-only
## view of a `T` does not live at all (R8 (g), E3021)": `on_hit` may change any `Player`, the one `p`
## reads included.

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None
    on_hit: fn() changes Player

    fn show() changes on_hit, Player:
        if this.target exists as p:
            this.on_hit()               #$ error[E3021]: `this.on_hit()` (line 16) may change any `Player`, this one included (`on_hit` is a `fn() changes Player`), and `p` reads a player and is used again on line 17; use `p` before the call: write `println(p.hp)` above line 16; or open the link again after it: `if this.target exists as q:` and `println(q.hp)` instead of line 17
            println(p.hp)

fn main() changes Player:
    player = Player()
    cam = Camera(on_hit=fn(): pass)
    cam.target = weakref player
    cam.show()
