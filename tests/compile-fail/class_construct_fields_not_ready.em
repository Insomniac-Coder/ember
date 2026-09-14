#$ test: compile-fail
#$ rules: CLS-1, CLS-2
#$ error[E1010]: class construction for `WithInit` is not implemented yet in this phase

class WithInit:
    fn init(mut self):
        pass

fn main():
    value = WithInit()
    println(1)
