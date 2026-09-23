#$ rules: DIA-12, MOD-2, MOD-3
#$ test: compile-fail
#$ error[E1010]: cannot find `support.hidden.pritn` in this scope
#$ not-help: did you mean `hidden.print`?
import support.hidden as hidden

fn main():
    hidden.pritn(1)
