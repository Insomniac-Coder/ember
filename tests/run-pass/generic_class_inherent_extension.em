#$ test: run-pass
#$ rules: TYP-16, IFC-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(em_6Holder3getG2c2a47066b4fc729d207037bb273fa35)

[T]
class Holder:
    value: T

extend[T] Holder[T]:
    fn get() -> T:
        return this.value

fn main():
    holder = Holder[i32](42)
    println(holder.get())
