#$ test: run-pass
#$ rules: OWN-8
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(em_Leaf_clone)
#$ assert-c: contains(em_Leaf_clone(_1.payload.Value._0))
#$ assert-c: contains(em_Entry_clone)

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
