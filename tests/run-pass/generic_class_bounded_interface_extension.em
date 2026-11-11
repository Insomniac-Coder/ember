#$ test: run-pass
#$ rules: TYP-16, TYP-17, IFC-1, TYP-22
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_7Measure em_vt_dyn_7Measure_for_6HolderGe6a28f706a47f67f0e77529bd802f82e)

interface Display:
    fn score() -> i32

interface Measure:
    fn measure() -> i32

struct Pixel implements Display:
    value: i32

    fn score() -> i32:
        return this.value

[T]
class Holder:
    value: T

extend[T: needs Display] Holder[T] implements Measure:
    fn measure() -> i32:
        return this.value.score()

fn measure(value: ref dyn Measure) -> i32:
    return value.measure()

fn main():
    holder = Holder[Pixel](Pixel(42))
    println(measure(ref holder))
