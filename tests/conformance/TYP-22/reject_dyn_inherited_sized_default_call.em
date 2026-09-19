#$ test: compile-fail
#$ profiles: debug, release, shipping
#$ rules: TYP-22

interface Base:
    fn inspect(self) -> i32 where Self: Sized:
        return 1

interface Child: Base:
    fn draw(self) -> i32

fn take(x: ref dyn Child) -> i32:
    return x.inspect() #$ error[E2020]: requires `Self: Sized`

fn main():
    pass
