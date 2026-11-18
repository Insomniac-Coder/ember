#$ test: run-pass
#$ rules: OWN-8, DRP-6
#$ profiles: debug, development, release
#$ stdout: 41

@derive(Clone)
class Counter:
    value: i32

    fn bump():
        this.value = this.value + 1

fn main():
    original = Counter(41)
    copy = original.clone()
    copy.bump()
    println(original.value)
