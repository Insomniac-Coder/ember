#$ test: compile-fail
#$ rules: R11, R3
## R11: "`copy of` a link is refused (E2116): another link to the same target is `weakref this.keep`, a
## copy of the target itself is `if this.keep exists as n:` then `copy of n`" (verdict 10; AN-65).

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    a = copy of cam.target              #$ error[E2116]: `copy of cam.target` would copy the link, not the player; for another link to the same player write `a = weakref cam.target`; for a copy of the player itself write `if cam.target exists as p:` and then `c = copy of p`
