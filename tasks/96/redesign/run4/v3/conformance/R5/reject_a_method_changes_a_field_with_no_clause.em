#$ test: compile-fail
#$ rules: R5
#$ help: fn open() changes angle:
## R5: a method whose first line says neither `changes` nor `owns this` only reads `this`; a change
## through it is a body that does not match its first line (AN-19, E2105).

class Door:
    angle: float = 0.0

    fn open():
        this.angle = 90.0    #$ error[E2105]: `open` changes `this.angle`; write its first line as `fn open() changes angle:`

fn main():
    d = Door()
    d.open()
    println(d.angle)
