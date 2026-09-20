#$ test: run-pass
#$ rules: OWN-8, DRP-6
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(ember_retain)

@derive(Clone)
class Counter:
    value: i32

    fn bump(mut self):
        self.value = self.value + 1

fn main():
    original = Counter(41)
    copy = original.clone()
    copy.bump()
    println(original.value)
