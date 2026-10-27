#$ test: run-pass
#$ rules: R5
#$ stdout: 45.0
## R5: a virtual's first line is the promise for its overrides and may say more than its own body
## (`pass` under `changes this`); the override says exactly what its body changes (AN-70's NEW).

open class Door:
    angle: float = 0.0

    virtual fn on_update(dt: float) changes this:
        pass

class SwingDoor(Door):
    override fn on_update(dt: float) changes angle:
        this.angle = min(this.angle + 90.0 * dt, 90.0)

fn main():
    d = SwingDoor()
    d.on_update(0.5)
    println(d.angle)
