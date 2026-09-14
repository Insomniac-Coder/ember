#$ test: compile-fail
#$ rules: CLS-7, EXC-1
#$ error[E1010]: mutable class-field access requires a `mut self` class method

class Inner:
    value: i32

class Outer:
    inner: Inner

    fn bad(mut self):
        self.inner.value = 1

fn main():
    value = Outer(Inner(0))
    value.bad()
