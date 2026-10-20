#$ test: run-pass
#$ command: lint
#$ rules: LNT-6, TYP-22
#$ stdout: 4.0
#$ stdout: 3.0
#$ stdout: 7.0
## `[LNT-6]` — `L4002`: a call through `dyn` whose receiver is always one final type, converted to
## `dyn` right here: a struct, or a class nothing derives from. A `dyn` value whose type the
## function does not know is dispatched as it must be.

interface Shape:
    fn area(self) -> f64

struct Square:
    side: f64

extend Square implements Shape:
    fn area(self) -> f64:
        return self.side * self.side

class Circle implements Shape:
    r: f64 = 1.0
    fn area(self) -> f64:
        return 3.0 * self.r * self.r

fn area_of(s: ref dyn Shape) -> f64:
    return s.area()

fn main():
    sq = Square(2.0)
    d: ref dyn Shape = ref sq
    println(d.area())  #$ warning[L4002]: this `Shape` call is dispatched dynamically, but its receiver is always a `Square`, a final type
    c: Shape = Circle()
    println(c.area())  #$ warning[L4002]: this `Shape` call is dispatched dynamically, but its receiver is always a `Circle`, a final type
    println(area_of(ref sq) + 3.0)
