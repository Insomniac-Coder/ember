#$ test: compile-fail
#$ rules: R5
#$ help: fn init():
## R5: `init` sets the fields of the object it builds through `this` with no word; `changes this` on
## it is refused (AN-32, E2108).

class Door:
    angle: float

    fn init() changes this:    #$ error[E2108]: `init` may always set the fields of the object it builds; write `fn init():`
        this.angle = 0.0

fn main():
    d = Door()
    println(d.angle)
