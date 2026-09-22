#$ test: compile-fail
#$ rules: CLS-9
#$ profiles: debug, release, shipping

class Version:
    let value: i32

    fn replace(mut self):
        self.value = 2      #$ error[E1010]: cannot assign to `let` field `Version.value` outside its `init`

    fn increment(mut self):
        self.value += 1     #$ error[E1010]: cannot assign to `let` field `Version.value` outside its `init`

fn main():
    version = Version(1)
    version.replace()
