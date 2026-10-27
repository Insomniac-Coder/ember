#$ test: compile-fail
#$ rules: CLS-4
#$ help: open class Door:
## design rules: R5
## R5: `virtual` lets subclasses replace a method; in a class that cannot be subclassed there are none,
## and `virtual` is refused (AN-70, E2112).

class Door:
    angle: float = 0.0

    virtual fn on_update(dt: float):    #$ error[E2112]: `Door` cannot be subclassed, so `virtual` on `on_update` changes nothing; to let subclasses replace it write `open class Door:`, or remove `virtual`
        pass

fn main():
    d = Door()
    d.on_update(0.5)
    println(d.angle)
