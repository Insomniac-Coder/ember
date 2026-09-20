#$ test: run-pass
#$ rules: TYP-16, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(em_Holder_bool_project__)

interface Project:
    fn project[U](self, value: U) -> U:
        return value

class Holder[T]:
    marker: T

extend[T] Holder[T] implements Project:
    pass

fn main():
    holder = Holder[bool](false)
    println(holder.project(42))
