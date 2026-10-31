#$ test: compile-fail
#$ rules: CLS-2, ENM-2
#$ error[E2100]: field `a` is not definitely initialized by class `init`
#$ error[E2100]: field `b` is not definitely initialized by class `init`

enum Kind:
    First
    Second

class Pair:
    a: i32
    b: i32

    fn init(kind: Kind):
        match kind:
            Kind.First:
                this.a = 1
            Kind.Second:
                this.b = 2

fn main():
    pair = Pair(Kind.First)
