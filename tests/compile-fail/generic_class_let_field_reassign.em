#$ test: compile-fail
#$ rules: CLS-9, TYP-16
#$ profiles: debug, release, shipping
# The mistake is in the generic method, so it is reported once, against
# `Version[T]` ([TYP-17], D-248), not per instance.

class Version[T]:
    let value: T

    fn replace(mut self, value: T):
        self.value = value      #$ error[E1010]: cannot assign to `let` field `Version[T].value` outside its `init`

fn main():
    version = Version[i32](1)
    version.replace(2)
