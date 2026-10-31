#$ test: compile-fail
#$ rules: CLS-9
#$ profiles: debug, development, release

class Version:
    let value: i32

    fn replace():
        this.value = 2      #$ error[E2104]: cannot assign to `let` field `Version.value` outside its `init`

    fn increment():
        this.value += 1     #$ error[E2104]: cannot assign to `let` field `Version.value` outside its `init`

fn main():
    version = Version(1)
    version.replace()
