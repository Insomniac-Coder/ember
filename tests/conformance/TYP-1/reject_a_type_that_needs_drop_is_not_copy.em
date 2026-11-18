#$ test: compile-fail
#$ rules: TYP-1
#$ error[E2040]: `String` does not implement `Copy`
## `[TYP-1]` - a heap type needs drop and is not `Copy`.

[T: needs Copy]
fn copy_it(x: T) -> T:
    return x

fn main():
    s = copy_it(String("hi"))
    println(s.len())
