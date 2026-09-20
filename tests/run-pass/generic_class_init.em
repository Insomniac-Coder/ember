#$ test: run-pass
#$ rules: TYP-16, CLS-2
#$ profiles: debug, release, shipping
#$ stdout: 42

class Holder[T]:
    value: T

    fn init(mut self, value: T):
        self.value = value

fn main():
    holder = Holder[i32](42)
    println(holder.value)
