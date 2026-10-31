#$ test: run-pass
#$ rules: FN-5, TYP-18
#$ stdout: 5 8 11 14
# Defaults on a generic owner's methods use substituted owner and method type
# parameters and may read the bound receiver or an earlier parameter.

[T: needs Copy]
struct Holder:
    value: T

    fn choose(other: T = this.value) -> T:
        return other

    [U: needs Copy]
    fn mirror(seed: U, other: U = seed) -> U:
        return other

fn main():
    holder = Holder[int](5)
    println(holder.choose(), holder.choose(8), holder.mirror(11), holder.mirror[int](13, 14))
