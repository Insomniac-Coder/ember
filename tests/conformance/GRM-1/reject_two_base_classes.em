#$ test: parse-fail
#$ rules: GRM-1
## `[GRM-1]` — a class has at most one base class.

open class A:
    pass

open class B:
    pass

class C(A, B):    #$ error[E0100]: a class has at most one base class
    pass

fn main():
    pass
