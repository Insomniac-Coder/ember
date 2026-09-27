#$ test: run-pass
#$ rules: FN-5, TYP-18
#$ stdout: 5 8 11 14
# Defaults on a generic owner's methods use substituted owner and method type
# parameters and may read the bound receiver or an earlier parameter.

struct Holder[T: Copy]:
    value: T

    fn choose(self, other: T = self.value) -> T:
        return other

    fn mirror[U: Copy](self, seed: U, other: U = seed) -> U:
        return other

fn main():
    holder = Holder[int](5)
    println(holder.choose(), holder.choose(8), holder.mirror(11), holder.mirror[int](13, 14))
