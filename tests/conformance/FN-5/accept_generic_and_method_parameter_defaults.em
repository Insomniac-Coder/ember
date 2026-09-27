#$ test: run-pass
#$ rules: FN-5, TYP-18
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ 9
#$ 12
#$ 22
# A generic default uses the concrete parameter type inferred from written
# arguments. Method defaults see the receiver and earlier parameter slots.

fn mirror[T: Copy](value: T, other: T = value) -> T:
    return other

struct Meter:
    total: int

extend Meter:
    fn add(mut self, delta: int = self.total, twice: int = delta * 2):
        self.total += twice

fn main():
    println(mirror(7))
    println(mirror[int](2, 9))
    meter = Meter(total=4)
    meter.add()
    println(meter.total)
    meter.add(delta=5)
    println(meter.total)
