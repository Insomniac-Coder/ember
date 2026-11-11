#$ test: run-pass
#$ rules: TYP-16, CLS-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(&em_ti_6HolderG2c2a47066b4fc729d207037bb273fa35))

[T]
class Holder:
    value: T

fn main():
    holder: Holder[i32] = Holder(42)
    println(holder.value)
