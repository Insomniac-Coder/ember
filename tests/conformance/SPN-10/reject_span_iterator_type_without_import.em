#$ test: compile-fail
#$ rules: SPN-10, MOD-2, TST-25

fn accepts(_iterator: SpanIter[i32]): #$ error[E1010]: cannot find type `SpanIter` in this scope
    pass

fn main():
    pass
