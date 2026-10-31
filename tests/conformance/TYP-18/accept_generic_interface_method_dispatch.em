#$ test: run-pass
#$ rules: TYP-17, TYP-18, TYP-22, MONO-1

interface Identity:
    [T]
    fn keep(value: T) -> T

struct IdentityImpl:
    marker: i32

extend IdentityImpl implements Identity:
    [U]
    fn keep(value: U) -> U:
        return value

[I: needs Identity, T]
fn through_identity(identity: I, value: T) -> T:
    return identity.keep(value)

fn main():
    identity = IdentityImpl(0)
    println(through_identity(identity, 48))
#$ stdout: 48
