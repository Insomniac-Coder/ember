#$ test: run-pass
#$ rules: R5, R8
#$ stdout: 45.0
## R5: the nearest allowed program: `Door.on_update` says `changes angle`, so a loop over `d.log` may
## call it; the override keeps to that line.

open class Door:
    angle: float = 0.0
    log: Array[String] = []

    virtual fn on_update(dt: float) changes angle:
        pass

class SwingDoor(Door):
    override fn on_update(dt: float) changes angle:
        this.angle = min(this.angle + 90.0 * dt, 90.0)

fn tick(ref mut d: Door):
    for line in d.log:
        d.on_update(0.5)

fn main():
    d: Door = SwingDoor()
    d.log.push(String("open"))
    tick(ref mut d)
    println(d.angle)
