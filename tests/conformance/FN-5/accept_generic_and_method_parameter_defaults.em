#$ test: run-pass
#$ rules: FN-5, TYP-18
#$ profiles: debug, development, release
#$ stdout: 7
#$ 9
#$ 12
#$ 22
# A generic default uses the concrete parameter type inferred from written
# arguments. Method defaults see the receiver and earlier parameter slots.

[T: needs Copy]
fn mirror(value: T, other: T = value) -> T:
    return other

struct Meter:
    total: int

extend Meter:
    fn add(delta: int = this.total, twice: int = delta * 2):
        this.total += twice

fn main():
    println(mirror(7))
    println(mirror[int](2, 9))
    meter = Meter(total=4)
    meter.add()
    println(meter.total)
    meter.add(delta=5)
    println(meter.total)
