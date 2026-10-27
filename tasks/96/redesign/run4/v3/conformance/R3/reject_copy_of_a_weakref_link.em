#$ test: compile-fail
#$ rules: R3, R11
#$ help: for another link to the same player write `a = weakref cam.target`; for a copy of the player itself write `if cam.target exists as p:` and then `c = copy of p`
## R3, R11: `copy of` is refused before a `weakref` link (E2116): it would copy the link, not the
## player, whose target may be gone; the message gives both lines (verdict 10; AN-65's message).

@derive(Clone)
class Player:
    health: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    a = copy of cam.target     #$ error[E2116]: `copy of cam.target` would copy the link, not the player
    println(player.health)
