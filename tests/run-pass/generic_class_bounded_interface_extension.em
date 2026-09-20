#$ test: run-pass
#$ rules: TYP-16, TYP-17, IFC-1, TYP-22
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_Measure em_vt_dyn_Measure_Holder_Pixel)

interface Display:
    fn score(self) -> i32

interface Measure:
    fn measure(self) -> i32

struct Pixel implements Display:
    value: i32

    fn score(self) -> i32:
        return self.value

class Holder[T]:
    value: T

extend[T: Display] Holder[T] implements Measure:
    fn measure(self) -> i32:
        return self.value.score()

fn measure(value: ref dyn Measure) -> i32:
    return value.measure()

fn main():
    holder = Holder[Pixel](Pixel(42))
    println(measure(ref holder))
