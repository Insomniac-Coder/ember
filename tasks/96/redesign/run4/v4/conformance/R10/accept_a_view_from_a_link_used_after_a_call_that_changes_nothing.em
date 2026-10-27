#$ test: compile-pass
#$ rules: R10, R8, R12
## R10 point 1: the nearest allowed program to
## `reject_a_view_from_a_link_used_after_a_call_that_may_change_players.em`: `on_hit` is a plain `fn()`,
## which changes nothing at all and so ends no `Player`, not even one its own state owns (R12, verdict
## 55: a function that may end a class object names its type); `p` lives across the call, and nothing
## runs for it while the program runs (section 2, "nothing").

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
