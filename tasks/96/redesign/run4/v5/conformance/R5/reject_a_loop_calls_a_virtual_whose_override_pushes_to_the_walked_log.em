#$ test: compile-fail
#$ rules: BRW-18, CTL-2
#$ help: update after the loop: move line 25 below the loop
## design rules: R5, R8
## R5, R8 (c), verdict 86: what a call through a `virtual` method changes is worked out from every
## override the program has (it replaces E2113, which compared an override with the virtual's first
## line): `SwingDoor.on_update` pushes to `this.log`, so `d.on_update(0.5)` inside a loop over `d.log` is
## refused (E3020), naming the override. An override that changes only `angle` is allowed there
## (`accept_a_loop_calls_a_virtual_that_changes_one_field`).

open class Door:
    angle: float = 0.0
    log: Array[String] = []

    virtual fn on_update(dt: float):
        pass

class SwingDoor(Door):
    override fn on_update(dt: float):
        this.angle = min(this.angle + 90.0 * dt, 90.0)
        this.log.push(String("swung"))

fn tick(ref mut d: Door):
    for line in d.log:
        println(line)
        d.on_update(0.5)       #$ error[E3020]: `d.on_update(0.5)` (line 25) may run `SwingDoor.on_update`, which pushes to `this.log` (line 20), while this loop walks `d.log` (line 23)

fn main():
    d: Door = SwingDoor()
    d.log.push(String("open"))
    tick(ref mut d)
    println(d.angle)
