#$ test: compile-fail
#$ profiles: debug, development, release
#$ rules: TYP-22

interface Base:
    fn inspect() -> i32 where Self: needs Sized:
        return 1

interface Child: needs Base:
    fn draw() -> i32

fn take(x: ref dyn Child) -> i32:
    return x.inspect() #$ error[E2020]: requires `Self: Sized`

fn main():
    pass
