#$ test: compile-fail
#$ rules: CLS-9, TYP-16
#$ profiles: debug, release, shipping

class Version[T]:
    let value: T

    fn replace(mut self, value: T):
        self.value = value      #$ error[E1010]: cannot assign to `let` field `Version_i32.value` outside its `init`

fn main():
    version = Version[i32](1)
    version.replace(2)
