#$ test: compile-fail
#$ rules: CLS-1, CLS-4
#$ error[E2020]: cannot instantiate abstract class `AbstractThing`

abstract class AbstractThing:
    pass

fn main():
    value = AbstractThing()
    println(1)
