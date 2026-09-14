#$ test: compile-pass
#$ rules: CLS-4
#$ warning[W2111]: `virtual` has no effect in a final class

class FinalThing:
    virtual fn tick(self):
        pass

fn main():
    println(1)
