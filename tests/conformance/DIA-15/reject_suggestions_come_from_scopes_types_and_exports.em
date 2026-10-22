#$ test: compile-fail
#$ rules: DIA-15, DIA-12
#$ error[E1010]: cannot find `countr` in this scope
#$ help: did you mean `counter`?
#$ error[E1010]: cannot find field `xpso` on `Point`
#$ help: did you mean `xpos`?
#$ error[E1010]: `Point` has no method named `length_squard`
#$ help: did you mean `length_squared`?
#$ error[E1010]: `Point` has no method named `scald`
#$ not-help: did you mean `scale`?
#$ error[E1010]: cannot find `sqrt` in this scope
#$ help: `sqrt` is exported by `std.math`: import it, `from std.math import sqrt`
#$ error[E1010]: cannot find type `Mutex` in this scope
#$ help: `Mutex` is exported by `std.sync`: import it, `from std.sync import Mutex`
# `[DIA-15]`: every suggestion comes from data the compiler already holds. A misspelt local is
# matched against the scope tables (`counter`); a misspelt field or method against the receiver's
# type, its fields and its method table (`xpos`, `length_squared`) — and only the receiver's: `scale`
# is a method of `Vector`, not of `Point`, so it is not offered for `p.scald()`, which would need
# checking the call against other types. A name another module exports is found in the index of
# exported names and offered as its import line (`sqrt` from `std.math`, the type `Mutex` from
# `std.sync`), of the right kind for the position (the case beside this one).

struct Point:
    xpos: i32
    ypos: i32

    fn length_squared(self) -> i32:
        return self.xpos * self.xpos + self.ypos * self.ypos

struct Vector:
    dx: i32

    fn scale(self) -> i32:
        return self.dx

fn guard(lock: Mutex[i32]):
    pass

fn main():
    counter = 1
    p = Point(1, 2)
    println(countr)
    println(p.xpso)
    println(p.length_squard())
    println(p.scald())
    println(sqrt(2.0))
