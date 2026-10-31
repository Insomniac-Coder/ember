#$ test: run-pass
#$ rules: TYP-16, IFC-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(em_6Holder7projectG572e1ef4d9565eac)

interface Project:
    [U]
    fn project(value: U) -> U:
        return value

[T]
class Holder:
    marker: T

extend[T] Holder[T] implements Project:
    pass

fn main():
    holder = Holder[bool](false)
    println(holder.project(42))
