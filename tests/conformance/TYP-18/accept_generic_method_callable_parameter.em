#$ test: run-pass
#$ rules: TYP-18, CLO-3, MONO-1

struct Apply:
    marker: i32

    fn call[T](self, value: T, transform: fn(T) -> T) -> T:
        return transform(value)

fn main():
    apply = Apply(0)
    println(apply.call(51, fn(value) => value))
#$ stdout: 51
