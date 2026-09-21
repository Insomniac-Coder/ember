#$ test: compile-fail
#$ rules: TYP-22, TYP-24, IFC-1
#$ profiles: debug, release, shipping
#$ error[E2070]: `describe` is offered by both `Left` and `Right`

interface Left:
    fn describe(self) -> i32

interface Right:
    fn describe(self) -> i32

struct Both:
    pass

extend Both implements Left:
    fn describe(self) -> i32:
        return 20

extend Both implements Right:
    fn describe(self) -> i32:
        return 22

fn call(value: ref dyn Left + Right) -> i32:
    return value.describe()

fn main():
    value = Both()
    println(call(ref value))
