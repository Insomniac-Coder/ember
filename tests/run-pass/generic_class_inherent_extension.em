#$ test: run-pass
#$ rules: TYP-16, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(em_Holder_i32_get)

class Holder[T]:
    value: T

extend[T] Holder[T]:
    fn get(self) -> T:
        return self.value

fn main():
    holder = Holder[i32](42)
    println(holder.get())
