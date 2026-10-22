#$ test: run-pass
#$ rules: OWN-8, BRW-8
#$ profiles: debug, development, release
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(em_4Leaf5clone)
#$ assert-c: contains(&(*self).payload.Value._0)
#$ assert-c: contains(em_5Entry5clone)

struct Leaf:
    value: i32

    fn clone(self) -> Leaf:
        return Leaf(self.value)

@derive(Clone)
enum Entry:
    Value(Leaf)
    Empty

fn duplicate[T: Clone](value: T) -> T:
    return value.clone()

fn main():
    original = Entry.Value(Leaf(42))
    copied = duplicate(original)
    match original:
        Entry.Value(value):
            println(value.value)
        Entry.Empty:
            println(0)
    match copied:
        Entry.Value(value):
            println(value.value)
        Entry.Empty:
            println(0)
