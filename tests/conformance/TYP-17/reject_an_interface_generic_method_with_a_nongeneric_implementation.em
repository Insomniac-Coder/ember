#$ test: compile-fail
#$ rules: IFC-1, TYP-17, TYP-18

interface Identity:
    fn keep[T](self, value: T) -> T

struct BadIdentity:
    marker: i32

extend BadIdentity implements Identity: #$ error[E2040]: `BadIdentity.keep` does not match the signature required by `Identity`
    fn keep(self, value: i32) -> i32:
        return value

fn main():
    value = BadIdentity(0)
    println(value.keep(55))
