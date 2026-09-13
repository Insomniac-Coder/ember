#$ test: run-pass
#$ rules: TYP-16, TYP-18, MONO-1

struct Picker:
    marker: i32

    fn pick[T](self, value: T) -> T:
        return value

fn main():
    picker = Picker(0)
    println(picker.pick(41))
    println(picker.pick[i64](42))
#$ stdout: 41
#$ 42
