#$ test: run-pass
#$ rules: FN-5
#$ profiles: debug, release, shipping
#$ stdout: hello, ann!
#$ hi, bo!
#$ hello, cy?
#$ yo, di!
#$ 1.5 6.0
#$ 19
#$ 99
#$ false true
# `[FN-5]` — a parameter the call leaves out takes its default, evaluated at
# each call in the callee's scope; named arguments may skip over one.

const LIMIT: int = 10

struct Meter:
    total: int

extend Meter:
    fn add(mut self, amount: int = 1, times: int = 1):
        self.total += amount * times

    fn make(start: int = LIMIT) -> Meter:
        return Meter(total=start)

fn greet(name: str, greeting: str = "hello", mark: String = "!") -> String:
    return f"{greeting}, {name}{mark}"

fn scale(x: f64, by: f64 = 0.5) -> f64:
    return x * by

fn show(x: Option[int] = None) -> bool:
    return x.is_some()

fn main():
    println(greet("ann"))
    println(greet("bo", "hi"))
    println(greet("cy", mark="?"))
    println(greet(greeting="yo", name="di"))
    println(scale(3.0), scale(3.0, 2.0))
    m = Meter.make()
    m.add()
    m.add(5)
    m.add(times=3)
    println(m.total)
    given = 99
    println(Meter.make(given).total)
    println(show(), show(5))
