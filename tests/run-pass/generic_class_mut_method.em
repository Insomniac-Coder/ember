#$ test: run-pass
#$ rules: TYP-16, CLS-1, CLS-7
#$ profiles: debug, release, shipping
#$ stdout: 42

class Counter[T]:
    value: i32
    marker: T

    fn bump(mut self):
        self.value = self.value + 1

fn main():
    counter = Counter[bool](41, false)
    counter.bump()
    println(counter.value)
