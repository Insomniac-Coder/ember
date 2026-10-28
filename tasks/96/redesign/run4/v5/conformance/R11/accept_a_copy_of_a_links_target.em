#$ test: compile-pass
#$ rules: OWN-8
## design rules: R11, R3
## R11: the other way out of `reject_copy_of_a_link.em`: open the link, then copy the player itself
## (verdict 10: the copy word copies the thing itself, never a link).

@derive(Clone)
class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    if cam.target exists as p:
        c = copy of p
        c.hp = 1
        println(c.hp, player.hp)
