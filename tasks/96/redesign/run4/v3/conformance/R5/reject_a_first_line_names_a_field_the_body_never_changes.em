#$ test: compile-fail
#$ rules: R5
#$ help: fn open() changes angle:
## R5: every clause is checked against the body both ways: a first line that names a field the body
## never changes is refused too, so the line stays exactly what the method does (E2105).

class Door:
    angle: float = 0.0
    hinge: int = 0

    fn open() changes angle, hinge:    #$ error[E2105]: `open` says `changes angle, hinge`, but it never changes `hinge`; write `fn open() changes angle:`
        this.angle = 90.0

fn main():
    d = Door()
    d.open()
    println(d.angle)
