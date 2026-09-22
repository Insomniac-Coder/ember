#$ test: run-pass
#$ rules: TYP-14, TYP-17, TYP-18, BRW-1
#$ profiles: debug, release, shipping
#$ stdout: 42

# The `ref mut T` receiver reads through to T for bound lookup while preserving
# the mutable receiver adjustment required by the selected method.
interface Bump:
    fn bump(mut self)

struct Counter:
    value: i32

extend Counter implements Bump:
    fn bump(mut self):
        self.value = self.value + 1

fn bump_once[T: Bump](mut value: ref mut T):
    value.bump()

fn main():
    counter = Counter(41)
    borrowed: ref mut Counter = ref mut counter
    bump_once[Counter](borrowed)
    println(borrowed.value)
