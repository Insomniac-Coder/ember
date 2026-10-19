#$ test: run-pass
#$ rules: TYP-16, CLS-1
#$ profiles: debug, development, release
#$ stdout: 42

class Holder[T]:
    value: T = 42

fn main():
    holder = Holder[i32]()
    println(holder.value)
