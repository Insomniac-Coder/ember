#$ test: compile-fail
#$ rules: CLS-2, CTL-3
#$ error[E2100]: field `value` is not definitely initialized by class `init`

class Holder:
    value: i32

    fn init(mut self):
        for i in 0..1:
            self.value = 42

fn main():
    holder = Holder()
