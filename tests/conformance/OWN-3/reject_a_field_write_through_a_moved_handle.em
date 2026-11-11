#$ test: compile-fail
#$ rules: OWN-3
#$ profiles: debug
# `[OWN-3]` — writing a field of a class instance reads the handle that names it, so once the handle
# has been moved away the write is a use after the move, `E3040`. (D-698: the write was taken to
# give the moved handle a value again, and the end of its scope released the object a second time.)

interface Speak:
    fn speak() -> int

class Mote implements Speak:
    n: int = 0

    fn speak() -> int:
        return this.n

fn main():
    m = Mote()
    boxed: owns dyn Speak = owns m
    m.n = 5    #$ error[E3040]: `m` was handed over on line 19 and is used on line 20; to keep it, write `owns copy of m`
    println(boxed.speak())
