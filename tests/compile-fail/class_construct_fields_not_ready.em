#$ test: compile-fail
#$ rules: CLS-1, CLS-2
#$ error[E2100]: field `value` is not definitely initialized by class `init`

class WithInit:
    value: i32

    fn init():
        pass

fn main():
    value = WithInit()
    println(1)
