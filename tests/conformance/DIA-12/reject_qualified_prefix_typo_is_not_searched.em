#$ rules: DIA-12, MOD-2, MOD-3
#$ test: compile-fail
#$ error[E1010]: `ioo` is not a module in scope
#$ not-help: did you mean `io::print`?
import support.io as io

fn main():
    ioo::print(1)
