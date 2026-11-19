#$ rules: DIA-12, MOD-2, MOD-3
#$ test: compile-fail
#$ error[E1010]: cannot find `support.io.pritn` in this scope
#$ help: the nearest name is `io.print`
import support.io as io

fn main():
    io.pritn(1)
