#$ test: compile-pass
#$ rules: TYP-22

interface Factory:
    fn clone(self) -> Self where Self: Sized:
        pass

fn take(x: ref dyn Factory):
    pass

fn main():
    pass
