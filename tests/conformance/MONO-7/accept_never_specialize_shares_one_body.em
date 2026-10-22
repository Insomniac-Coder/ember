#$ test: run-pass
#$ rules: MONO-5, MONO-7, MONO-8, MONO-9
#$ profiles: debug, development, release
#$ assert-c-count: contains("(ember_dyn x, int64_t count) {") == 1
#$ assert-c: contains("->slot0)(x.data)")
#$ assert-c: contains("->slot1)(x.data)")
#$ stdout: 4.0 4 1
#$ stdout: 3.0 0 1
#$ stdout: 4.0 4 1
## `[MONO-7]` — `@never_specialize` makes one body that every instance shares: each of
## `describe[Square]` and `describe[Circle]` passes its value as `ref dyn Shape` to the shared body,
## which calls `area` and `sides` through the table: slots 0 and 1 (`[MONO-8]`: one indirect call per
## bound-method call, the results specialised code gives, nothing allocated for the table). The
## shared body is one symbol (`[MONO-9]`): declared and defined once, called by both instances. `x`
## is used only as the receiver of calls to `Shape`'s methods, so the generic is shareable
## (`[MONO-5]`).

interface Shape:
    fn area(self) -> f64
    fn sides(self) -> int

struct Square:
    side: f64

extend Square implements Shape:
    fn area(self) -> f64:
        return self.side * self.side
    fn sides(self) -> int:
        return 4

class Circle implements Shape:
    r: f64 = 1.0
    fn area(self) -> f64:
        return 3.0 * self.r * self.r
    fn sides(self) -> int:
        return 0

@never_specialize
fn describe[T: Shape](x: T, count: int) -> String:
    return f"{x.area()} {x.sides()} {count}"

fn main():
    println(describe(Square(2.0), 1))
    println(describe(Circle(), 1))
    println(describe(Square(2.0), 1))
