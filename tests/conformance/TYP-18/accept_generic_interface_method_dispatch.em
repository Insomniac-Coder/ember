#$ test: run-pass
#$ rules: TYP-17, TYP-18, TYP-22, MONO-1

interface Identity:
    fn keep[T](self, value: T) -> T

struct IdentityImpl:
    marker: i32

extend IdentityImpl implements Identity:
    fn keep[U](self, value: U) -> U:
        return value

fn through_identity[I: Identity, T](identity: I, value: T) -> T:
    return identity.keep(value)

fn main():
    identity = IdentityImpl(0)
    println(through_identity(identity, 48))
#$ stdout: 48
