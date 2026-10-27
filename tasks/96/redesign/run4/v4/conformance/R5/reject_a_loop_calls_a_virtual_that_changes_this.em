#$ test: compile-fail
#$ rules: R5, R8
#$ help: d.on_update(0.5)
## R5: a call through a `virtual` method is checked against the virtual's first line, not the
## override's: `Door.on_update` says `changes this`, so it may change `d.log` while a loop walks it
## (R8 (c), E3020), whatever `SwingDoor`'s override changes.

open class Door:
    angle: float = 0.0
    log: Array[String] = []

    virtual fn on_update(dt: float) changes this:
        pass

class SwingDoor(Door):
    override fn on_update(dt: float) changes angle:
        this.angle = min(this.angle + 90.0 * dt, 90.0)

fn tick(ref mut d: Door):
    for line in d.log:
        d.on_update(0.5)    #$ error[E3020]: `on_update` may change every field of `Door`, `log` included (`Door.on_update` says `changes this`), and this loop walks `d.log` (line 20); call `d.on_update(0.5)` after the loop

fn main():
    d: Door = SwingDoor()
    d.log.push(String("open"))
    tick(ref mut d)
    println(d.angle)
