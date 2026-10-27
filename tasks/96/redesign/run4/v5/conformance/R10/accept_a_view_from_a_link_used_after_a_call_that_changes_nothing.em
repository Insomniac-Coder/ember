#$ test: compile-pass
#$ rules: BRW-17
## design rules: R10, R8, R12
## R10 point 1: the nearest allowed program to
## `reject_a_view_from_a_link_used_after_a_call_that_may_change_players.em`: the only function the
## program stores in `on_hit`, `fn(): pass`, changes nothing and so ends no `Player` (R12, verdict 86:
## what a call through a function value does is worked out from what is stored in it); `p` lives across
## the call, and nothing runs for it while the program runs (section 2, "nothing").

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None
    on_hit: fn()

    fn show():
        if this.target exists as p:
            this.on_hit()
            println(p.hp)

fn main():
    player = Player()
    cam = Camera(on_hit=fn(): pass)
    cam.target = weakref player
    cam.show()
