#$ test: compile-fail
#$ rules: CLS-7, EXC-1
#$ error[E1010]: mutable class-field access requires a `mut self` class method

class ReadOnly:
    value: i32

    fn bad(self):
        self.value = 1

fn main():
    value = ReadOnly(0)
    value.bad()
