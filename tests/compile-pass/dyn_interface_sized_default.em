#$ test: compile-pass
#$ rules: TYP-22

interface Factory:
    fn clone(self) -> Self where Self: Sized:
        todo()
    fn draw(self) -> i32

fn take(x: ref dyn Factory) -> i32:
    return x.draw()

fn main():
    pass
