#$ test: compile-fail
#$ rules: R13, R11
#$ help: instead of the `match` line and its two branches, write `if cam.target exists as p:` with `println(p.health)`, and `else:` with `println("gone")`
## R13, R11 and the removed words of `V4/rules.md` section 3: Rust's `upgrade` is written as an
## opening, `if w exists as p:` (E1010; AN-65). The nearest program that is allowed is
## `accept_an_opened_name_that_is_new`.

class Player:
    health: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    match cam.target.upgrade():    #$ error[E1010]: a `weakref` link is opened with `exists as`, not with `upgrade`
        Some(p):
            println(p.health)
        None:
            println("gone")
