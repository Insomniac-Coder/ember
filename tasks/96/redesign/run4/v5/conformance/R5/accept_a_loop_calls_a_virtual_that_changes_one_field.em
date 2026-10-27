#$ test: run-pass
#$ rules: R5, R8
#$ stdout: 45.0
## R5, R8 (c), verdict 86: what a call through a `virtual` changes is worked out from every override
## the program has: `SwingDoor.on_update` changes only `angle`, so a loop over `d.log` may call it (the
## nearest allowed program to `reject_a_loop_calls_a_virtual_whose_override_pushes_to_the_walked_log`).

open class Door:
    angle: float = 0.0
    log: Array[String] = []

    virtual fn on_update(dt: float):
        pass

class SwingDoor(Door):
    override fn on_update(dt: float):
        this.angle = min(this.angle + 90.0 * dt, 90.0)

fn tick(ref mut d: Door):
    for line in d.log:
        d.on_update(0.5)

fn main():
    d: Door = SwingDoor()
    d.log.push(String("open"))
    tick(ref mut d)
    println(d.angle)
