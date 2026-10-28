#$ test: run-pass
#$ rules: FN-4
#$ stdout: 90.0
## design rules: R5
## R5, verdict 86: a method changes its object through `this` with no word on its first line; what it
## changes (`angle`) is worked out from its body, and the call on the object carries no word (AN-19).

class Door:
    angle: float = 0.0

    fn open():
        this.angle = 90.0

fn main():
    d = Door()
    d.open()
    println(d.angle)
