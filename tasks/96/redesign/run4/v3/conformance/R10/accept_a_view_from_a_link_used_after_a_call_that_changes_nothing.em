#$ test: compile-pass
#$ rules: R10, R8, R12
## R10 point 1: the nearest allowed program: `on_hit` is a plain `fn()`, which changes nothing outside
## itself and so ends no `Player`; `p` lives across the call and costs nothing.

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None
    on_hit: fn()

    fn show() changes on_hit:
        if this.target exists as p:
            this.on_hit()
            println(p.hp)

fn main():
    player = Player()
    cam = Camera(on_hit=fn(): pass)
    cam.target = weakref player
    cam.show()
