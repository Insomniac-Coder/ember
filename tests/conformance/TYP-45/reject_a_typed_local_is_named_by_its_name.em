#$ test: compile-fail
#$ rules: TYP-45, OWN-10
## `[OWN-10]` - `E2123` for a typed local names the local (`b`), not its annotation run together.

interface Shape:
    fn area() -> int

class Sq implements Shape:
    s: int

    fn area() -> int:
        return this.s * this.s

fn main():
    b: owns dyn Shape = Sq(3)    #$ error[E2123]: `b` owns its shape; say so where it is made: `b: owns dyn Shape = owns Sq(3)`
    println(b.area())
