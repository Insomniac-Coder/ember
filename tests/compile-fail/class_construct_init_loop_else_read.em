#$ test: compile-fail
#$ rules: CLS-2, CTL-3, CTL-5
#$ error[E2100]: field `first` is read before it is initialized
#$ error[E2100]: field `first` is not definitely initialized by class `init`

class LoopElseRead:
    first: i32
    second: i32

    fn init(mut self):
        while false:
            self.first = 1
        else:
            self.second = self.first

fn main():
    value = LoopElseRead()
