#$ test: run-pass
#$ rules: FN-4
#$ stdout: 45.0
## design rules: R5
## R5, verdict 86: an override may change what its virtual's body leaves alone: no first line says what
## either changes, and a call through the virtual counts what every override changes (AN-70's NEW; the
## E2113 refusal of an override that changed more than its virtual's line is gone).

open class Door:
    angle: float = 0.0

    virtual fn on_update(dt: float):
        pass

class SwingDoor(Door):
    override fn on_update(dt: float):
        this.angle = min(this.angle + 90.0 * dt, 90.0)

fn main():
    d = SwingDoor()
    d.on_update(0.5)
    println(d.angle)
