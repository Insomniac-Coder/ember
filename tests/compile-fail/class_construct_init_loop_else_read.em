#$ test: compile-fail
#$ rules: CLS-2, CTL-3, CTL-5
#$ error[E2100]: field `first` is read before it is initialized
#$ error[E2100]: field `first` is not definitely initialized by class `init`

class LoopElseRead:
    first: i32
    second: i32

    fn init():
        while false:
            this.first = 1
        else:
            this.second = this.first

fn main():
    value = LoopElseRead()
