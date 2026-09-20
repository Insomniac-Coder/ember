#$ test: run-pass
#$ rules: OWN-8, TYP-16, TYP-17, DRP-6
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(ember_retain)

@derive(Clone)
class Counter[T]:
    value: i32
    payload: T

    fn bump(mut self):
        self.value = self.value + 1

fn duplicate[T: Clone](value: T) -> T:
    return value.clone()

struct Marker:
    value: i32

fn main():
    original = Counter[Marker](41, Marker(0))
    copy = duplicate(original)
    copy.bump()
    println(original.value)
