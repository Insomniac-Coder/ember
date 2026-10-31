#$ test: compile-fail
#$ rules: CLS-2, CTL-3
#$ error[E2100]: field `value` is not definitely initialized by class `init`

class Holder:
    value: i32

    fn init():
        for i in 0..1:
            this.value = 42

fn main():
    holder = Holder()
