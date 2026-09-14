#$ test: compile-fail
#$ rules: CLS-2, CTL-5
#$ error[E2100]: field `value` is not definitely initialized by class `init`

class Holder:
    value: i32

    fn init(mut self, ready: bool):
        while ready:
            self.value = 42

fn main():
    holder = Holder(true)
