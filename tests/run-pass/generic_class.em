#$ test: run-pass
#$ rules: TYP-16, CLS-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(ember_obj_new(&em_ti_Holder_i32))

class Holder[T]:
    value: T

fn main():
    holder: Holder[i32] = Holder(42)
    println(holder.value)
