#$ test: run-pass
#$ rules: TYP-16, TYP-18, MONO-1

[T]
struct Holder:
    value: T

    [U]
    fn choose(other: U) -> U:
        return other

    [U]
    fn original(_ignored: U) -> T:
        return this.value

fn main():
    holder = Holder[i32](1)
    println(holder.original(0i64))
    println(holder.choose(43))
    println(holder.choose[i64](44))
#$ stdout: 1
#$ 43
#$ 44
