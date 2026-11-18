#$ test: run-pass
#$ rules: OWN-8, TYP-16, TYP-17, DRP-6
#$ profiles: debug, development, release
#$ stdout: 41

@derive(Clone)
[T]
class Counter:
    value: i32
    payload: T

    fn bump():
        this.value = this.value + 1

[T: needs Clone]
fn duplicate(value: T) -> T:
    return copy of value

struct Marker:
    value: i32

fn main():
    original = Counter[Marker](41, Marker(0))
    copy = duplicate(original)
    copy.bump()
    println(original.value)
