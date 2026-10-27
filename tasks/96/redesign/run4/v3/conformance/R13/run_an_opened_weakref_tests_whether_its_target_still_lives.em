#$ test: run-pass
#$ rules: R13, R11
#$ stdout: gone
#$ 100
#$ gone
## R13, R11 and `V3/rules.md` section 2 ("one test of the target's header, each read of a `weakref`
## link"): `if cam.target exists as p:` runs its block only while the player still lives, and `else:`
## when the link holds no target or its target has ended; nothing stops the program (AN-65). Verified
## by its output and a clean AddressSanitizer run (the ended player's block is kept by the link's weak
## count, so the test reads no freed memory).

import std.mem

class Player:
    health: int = 100

class Camera:
    target: weakref Player = None

fn show(cam: Camera):
    if cam.target exists as p:
        println(p.health)
    else:
        println("gone")

fn main():
    cam = Camera()
    show(cam)
    player = Player()
    cam.target = weakref player
    show(cam)
    mem.drop(player)
    show(cam)
