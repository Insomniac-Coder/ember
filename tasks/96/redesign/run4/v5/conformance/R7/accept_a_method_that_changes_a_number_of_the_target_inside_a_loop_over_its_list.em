#$ test: run-pass
#$ rules: BRW-18, CTL-2
#$ stdout: 99
## design rules: R7, R8, R5
## R7, R8 (c): the nearest allowed program to
## `reject_a_method_that_may_change_the_target_inside_a_loop_over_it.em`: the compiler works out from
## `shake`'s body that it writes only `hp` of the player `cam.target` links to (R5, verdict 86), a
## number, not the walked list, so `cam.shake()` may run inside the loop, as `cam.target.hp -= 1`
## written there may. AN-48's `shake`.

struct Player:
    hp: int
    inventory: Array[String]

struct Camera:
    target: sharedref mut Player

    fn shake():
        this.target.hp -= 1

struct Hud:
    target: sharedref mut Player

fn main():
    p = Player(hp=100, inventory=[String("key")])
    cam = Camera(target=sharedref mut p)
    hud = Hud(target=sharedref mut p)
    for _ in hud.target.inventory:
        cam.shake()
    println(p.hp)
