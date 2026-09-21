#$ test: run-pass
#$ rules: OWN-8, TYP-16, ENM-1, MONO-1
#$ stdout: 3
#$ stdout: 3

@derive(Copy)
enum Flag[T]:
    Value(value: T)

fn main():
    original = Flag[i32].Value(3)
    copied = original
    match original:
        Flag.Value(value): println(value)
    match copied:
        Flag.Value(value): println(value)
