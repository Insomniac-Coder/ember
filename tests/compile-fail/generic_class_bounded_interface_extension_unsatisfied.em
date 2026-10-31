#$ test: compile-fail
#$ rules: TYP-16, TYP-17, IFC-1, TYP-22
#$ error[E2020]: expected `ref dyn Measure`, found `ref Holder[Plain]`

interface Display:
    fn score() -> i32

interface Measure:
    fn measure() -> i32

struct Plain:
    value: i32

[T]
class Holder:
    value: T

extend[T: needs Display] Holder[T] implements Measure:
    fn measure() -> i32:
        return this.value.score()

fn measure(value: ref dyn Measure) -> i32:
    return value.measure()

fn main():
    holder = Holder[Plain](Plain(42))
    println(measure(ref holder))
