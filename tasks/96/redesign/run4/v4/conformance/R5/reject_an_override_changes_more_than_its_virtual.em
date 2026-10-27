#$ test: compile-fail
#$ rules: R5, R12
#$ help: virtual fn on_update(dt: float) changes this:
## R5: a call through a `virtual` method is checked against the virtual's first line, so an override
## may change no more than that line says (AN-70, E2113).

open class Door:
    angle: float = 0.0

    virtual fn on_update(dt: float):
        pass

class SwingDoor(Door):
    override fn on_update(dt: float) changes angle:    #$ error[E2113]: `SwingDoor.on_update` changes `angle`, but it overrides `Door.on_update`, whose first line changes nothing, and callers of `Door` rely on that; write `virtual fn on_update(dt: float) changes this:` in `Door`
        this.angle = min(this.angle + 90.0 * dt, 90.0)

fn main():
    d = SwingDoor()
    d.on_update(0.5)
    println(d.angle)
