#$ test: compile-fail
#$ rules: R5
#$ help: open class Door:
## R5: a virtual's first line is a promise to overrides; in a class that cannot be subclassed there are
## none, and `virtual` is refused (AN-70, E2112).

class Door:
    angle: float = 0.0

    virtual fn on_update(dt: float) changes this:    #$ error[E2112]: `Door` cannot be subclassed, so `virtual` on `on_update` changes nothing; to let subclasses replace it write `open class Door:`, or remove `virtual`
        pass

fn main():
    d = Door()
    d.on_update(0.5)
    println(d.angle)
