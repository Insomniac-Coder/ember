#$ rules: DIA-12, MOD-2, MOD-3
#$ test: compile-fail
#$ error[E1010]: cannot find `support.candidates.pritn` in this scope
#$ help: did you mean `picks::prin`?
#$ help: did you mean `picks::print`?
#$ help: did you mean `picks::prit`?
#$ not-help: did you mean `picks::prtin`?
import support.candidates as picks

fn main():
    picks::pritn(1)
