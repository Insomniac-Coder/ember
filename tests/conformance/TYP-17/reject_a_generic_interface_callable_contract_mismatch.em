#$ test: compile-fail
#$ rules: IFC-1, TYP-17, TYP-18, CLO-3

interface Transform:
    fn apply[T](self, value: T, transform: fn(T) -> T) -> T

struct BadTransform:
    marker: i32

extend BadTransform implements Transform: #$ error[E2040]: `BadTransform.apply` does not match the signature required by `Transform`
    fn apply[U](self, value: U, transform: fn(i32) -> i32) -> U:
        return value

fn main():
    value = BadTransform(0)
    println(value.apply(58, fn(item) => item))
