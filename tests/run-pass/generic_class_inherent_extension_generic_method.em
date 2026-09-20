#$ test: run-pass
#$ rules: TYP-16, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 42

class Holder[T]:
    value: T

extend[T] Holder[T]:
    fn replace[Value](self, value: Value) -> Value:
        return value

fn main():
    holder = Holder[bool](false)
    println(holder.replace(42))
