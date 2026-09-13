#$ test: run-pass
#$ rules: TYP-16, TYP-18, MONO-1

struct Holder[T]:
    value: T

    fn choose[U](self, other: U) -> U:
        return other

    fn original[U](self, _ignored: U) -> T:
        return self.value

fn main():
    holder = Holder[i32](1)
    println(holder.original(0i64))
    println(holder.choose(43))
    println(holder.choose[i64](44))
#$ stdout: 1
#$ 43
#$ 44
