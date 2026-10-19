#$ test: compile-fail
#$ rules: OWN-3
#$ profiles: debug
# `[OWN-3]` — writing a field of a class instance reads the handle that names it, so once the handle
# has been moved away the write is a use after the move, `E3040`. (D-698: the write was taken to
# give the moved handle a value again, and the end of its scope released the object a second time.)

interface Speak:
    fn speak(self) -> int

class Mote implements Speak:
    n: int = 0

    fn speak(self) -> int:
        return self.n

fn main():
    m = Mote()
    boxed: Box[dyn Speak] = Box(m)
    m.n = 5    #$ error[E3040]: `m` has been moved out of
    println(boxed.speak())
