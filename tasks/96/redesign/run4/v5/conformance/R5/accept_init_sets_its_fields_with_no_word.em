#$ test: run-pass
#$ rules: R5
#$ stdout: 0.0
#$ stdout: 90.0
## R5: `init` sets its object's fields through `this` with no word, and `open` changes `angle` with no
## word either (verdict 86; AN-32's NEW).

class Door:
    angle: float

    fn init():
        this.angle = 0.0

    fn open():
        this.angle = 90.0

fn main():
    d = Door()
    println(d.angle)
    d.open()
    println(d.angle)
