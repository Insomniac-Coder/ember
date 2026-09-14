#$ test: compile-fail
#$ rules: CLS-1, CLS-2, CLS-3, CLS-6
#$ error[E1010]: default expression for class field `value` is not implemented yet in this phase

fn default_value() -> i32:
    return 3

class Defaults:
    value: i32 = default_value()

    fn init(mut self):
        pass

fn main():
    value = Defaults()
