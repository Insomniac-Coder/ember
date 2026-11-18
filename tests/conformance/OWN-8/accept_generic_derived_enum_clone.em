#$ test: run-pass
#$ rules: OWN-8, TYP-16, ENM-1, MONO-1
#$ stdout: 9

@derive(Clone)
enum Entry[T]:
    Value(value: T)

fn main():
    original = Entry[i32].Value(9)
    cloned = copy of original
    match cloned:
        Entry.Value(value): println(value)
