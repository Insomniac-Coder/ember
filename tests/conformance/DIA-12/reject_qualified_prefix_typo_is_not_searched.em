#$ rules: DIA-12, MOD-2, MOD-3
#$ test: compile-fail
#$ error[E1010]: cannot find `ioo` in this scope
#$ not-help: `io.print`
import support.io as io

fn main():
    ioo.print(1)
