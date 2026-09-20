#$ test: run-pass
#$ rules: TYP-16, CLS-1
#$ profiles: debug, release, shipping
#$ stdout: 42

class Holder[T]:
    value: T

    fn get(self) -> T:
        return self.value

fn main():
    holder = Holder[i32](42)
    println(holder.get())
