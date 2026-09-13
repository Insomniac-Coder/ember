#$ test: run-pass
#$ rules: TYP-17, TYP-18, TYP-22, MONO-1

interface IdentityFactory:
    fn keep[T](value: T) -> T

struct FactoryImpl:
    marker: i32

extend FactoryImpl implements IdentityFactory:
    fn keep[U](value: U) -> U:
        return value

fn through_factory[F: IdentityFactory, T](value: T) -> T:
    return F.keep(value)

fn main():
    println(through_factory[FactoryImpl, i32](49))
#$ stdout: 49
