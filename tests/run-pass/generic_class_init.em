#$ test: run-pass
#$ rules: TYP-16, CLS-2
#$ profiles: debug, development, release
#$ stdout: 42

[T]
class Holder:
    value: T

    fn init(value: T):
        this.value = value

fn main():
    holder = Holder[i32](42)
    println(holder.value)
