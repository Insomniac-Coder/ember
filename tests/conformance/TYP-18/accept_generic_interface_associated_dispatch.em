#$ test: run-pass
#$ rules: TYP-17, TYP-18, TYP-22, MONO-1

interface IdentityFactory:
    [T]
    static fn keep(value: T) -> T

struct FactoryImpl:
    marker: i32

extend FactoryImpl implements IdentityFactory:
    [U]
    static fn keep(value: U) -> U:
        return value

[F: needs IdentityFactory, T]
fn through_factory(value: T) -> T:
    return F.keep(value)

fn main():
    println(through_factory[FactoryImpl, i32](49))
#$ stdout: 49
