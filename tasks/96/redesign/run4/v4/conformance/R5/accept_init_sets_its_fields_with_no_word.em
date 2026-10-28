#$ test: run-pass
#$ rules: R5
#$ stdout: 0.0
#$ stdout: 90.0
## R5: `init` sets its object's fields through `this` with no word; `open` names the field it changes
## (AN-32's NEW).

class Door:
    angle: float

    fn init():
        this.angle = 0.0

    fn open() changes angle:
        this.angle = 90.0

fn main():
    d = Door()
    println(d.angle)
    d.open()
    println(d.angle)
