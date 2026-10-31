#$ test: run-pass
#$ rules: TYP-16, IFC-1
#$ profiles: debug, development, release
#$ stdout: 42

[T]
class Holder:
    value: T

extend[T] Holder[T]:
    [Value]
    fn replace(value: Value) -> Value:
        return value

fn main():
    holder = Holder[bool](false)
    println(holder.replace(42))
