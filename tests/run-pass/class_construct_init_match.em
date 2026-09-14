#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-3, ENM-2
#$ stdout: 42

enum Kind:
    Answer
    Other

class Holder:
    value: i32

    fn init(mut self, kind: Kind):
        match kind:
            Kind.Answer:
                self.value = 42
            Kind.Other:
                self.value = 0

fn main():
    holder = Holder(Kind.Answer)
    println(holder.value)
