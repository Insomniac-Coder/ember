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

    fn init(mut self, kind: Kind):
        match kind:
            Kind.First:
                self.a = 1
            Kind.Second:
                self.b = 2

fn main():
    pair = Pair(Kind.First)
