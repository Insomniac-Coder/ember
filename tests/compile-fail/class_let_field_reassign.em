#$ test: compile-fail
#$ rules: CLS-9
#$ profiles: debug, development, release

class Version:
    let value: i32

    fn replace():
        this.value = 2      #$ error[E2104]: `value` is `let`: it is set in `init` and never changed after; to change it, write the field without `let`: `value: i32`

    fn increment():
        this.value += 1     #$ error[E2104]: `value` is `let`: it is set in `init` and never changed after; to change it, write the field without `let`: `value: i32`

fn main():
    version = Version(1)
    version.replace()
