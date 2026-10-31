#$ test: run-pass
#$ rules: TYP-16, CLS-1
#$ profiles: debug, development, release
#$ stdout: 42

[T]
class Holder:
    value: T

    fn get() -> T:
        return this.value

fn main():
    holder = Holder[i32](42)
    println(holder.get())
