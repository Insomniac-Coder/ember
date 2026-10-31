#$ test: run-pass
#$ rules: TYP-16, CLS-1, CLS-7
#$ profiles: debug, development, release
#$ stdout: 42

[T]
class Counter:
    value: i32
    marker: T

    fn bump():
        this.value = this.value + 1

fn main():
    counter = Counter[bool](41, false)
    counter.bump()
    println(counter.value)
