#$ test: run-pass
#$ rules: CLO-10, CLO-13
#$ profiles: debug, development, release
#$ stdout: 1
#$ 2
#$ 3
# `[CLO-10]` — an owned callable value holds its own copy of the state it captured. Bound to a local
# and then stored in a class's field, the closure copies the local's state, so the class handle it
# captured has one more owner: the local's end releases its own, and the field's value keeps the
# object alive until the `Ticker` ends (D-696: the copy did not retain, and the object was released
# twice).

class Particle:
    x: int = 0
    step: int = 1

class Ticker:
    f: fn() -> int

    fn tick(self) -> int:
        return self.f()

fn make() -> Ticker:
    p = Particle()
    step = owned fn() -> int:
        p.x = p.x + p.step
        return p.x
    return Ticker(step)

@noinline
fn made_apart() -> Ticker:
    return make()

fn main():
    t = made_apart()
    println(t.tick())
    println(t.tick())
    println(t.tick())
