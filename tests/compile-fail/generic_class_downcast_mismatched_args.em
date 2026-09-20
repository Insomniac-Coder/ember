#$ test: compile-fail
#$ rules: TYP-16, DSP-4, OBJ-2
#$ profiles: debug, release, shipping
#$ error[E2020]: cannot downcast `Child_bool` to unrelated class `Child_i32`

class Child[T]:
    value: T

fn main():
    value = Child[bool](true)
    bad: Option[Child[i32]] = value as? Child[i32]
