#$ test: compile-fail
#$ rules: TYP-16, TYP-17, IFC-1, TYP-22
#$ error[E2020]: expected `ref dyn Measure`, found `ref Holder_Plain`

interface Display:
    fn score(self) -> i32

interface Measure:
    fn measure(self) -> i32

struct Plain:
    value: i32

class Holder[T]:
    value: T

extend[T: Display] Holder[T] implements Measure:
    fn measure(self) -> i32:
        return self.value.score()

fn measure(value: ref dyn Measure) -> i32:
    return value.measure()

fn main():
    holder = Holder[Plain](Plain(42))
    println(measure(ref holder))
