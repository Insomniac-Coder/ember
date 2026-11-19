#$ rules: DIA-12, MOD-2, MOD-3
#$ test: compile-fail
#$ error[E1010]: cannot find `support.io.pritn` in this scope
#$ help: the nearest name is `support.io.print`
import support

fn main():
    support.io.pritn(1)
