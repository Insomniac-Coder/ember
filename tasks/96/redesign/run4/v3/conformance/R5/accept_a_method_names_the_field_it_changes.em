#$ test: run-pass
#$ rules: R5
#$ stdout: 90.0
## R5, row 3: `changes angle` lets the method change exactly that field; the call on the object carries
## no word (AN-19).

class Door:
    angle: float = 0.0

    fn open() changes angle:
        this.angle = 90.0

fn main():
    d = Door()
    d.open()
    println(d.angle)
