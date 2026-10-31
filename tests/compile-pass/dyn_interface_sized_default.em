#$ test: compile-pass
#$ rules: TYP-22

interface Factory:
    fn clone() -> Self where Self: needs Sized:
        todo()
    fn draw() -> i32

fn take(x: ref dyn Factory) -> i32:
    return x.draw()

fn main():
    pass
