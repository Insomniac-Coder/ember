#$ test: compile-fail
#$ rules: TYP-16, OBJ-2
#$ profiles: debug, development, release
#$ error[E2020]: `is` requires related class handles, found `Child[bool]` and `Child[i32]`

[T]
class Child:
    value: T

fn main():
    first = Child[bool](true)
    other = Child[i32](42)
    result = first is other
    println(result)
