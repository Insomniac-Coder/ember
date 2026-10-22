#$ test: run-pass
#$ rules: TYP-16, IFC-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(em_6Holder3getG2c2a47066b4fc729)

class Holder[T]:
    value: T

extend[T] Holder[T]:
    fn get(self) -> T:
        return self.value

fn main():
    holder = Holder[i32](42)
    println(holder.get())
