#$ test: compile-fail
#$ rules: CLS-9, TYP-16
#$ profiles: debug, development, release
# The mistake is in the generic method, so it is reported once, against
# `Version[T]` ([TYP-17], D-248), not per instance.

[T]
class Version:
    let value: T

    fn replace(value: T):
        this.value = value      #$ error[E2104]: `value` is `let`: it is set in `init` and never changed after; to change it, write the field without `let`: `value: T`

fn main():
    version = Version[i32](1)
    version.replace(2)
