#$ test: compile-fail
#$ rules: TYP-18

struct Picker:
    marker: i32

    fn pick[T](self, value: T) -> T:
        return value

fn main():
    picker = Picker(0)
    println(picker.pick[i32, i64](51)) #$ error[E2020]: `pick` takes 1 type arguments, found 2
