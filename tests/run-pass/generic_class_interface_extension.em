#$ test: run-pass
#$ rules: TYP-16, IFC-1, TYP-22
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_7Measure em_vt_dyn_7Measure_for_6HolderGda5cc12a188c7619)

interface Measure:
    fn measure(self) -> i32

class Holder[T]:
    value: i32
    marker: T

extend[T] Holder[T] implements Measure:
    fn measure(self) -> i32:
        return self.value

fn measure(value: ref dyn Measure) -> i32:
    return value.measure()

fn main():
    holder = Holder[bool](42, false)
    println(measure(ref holder))
