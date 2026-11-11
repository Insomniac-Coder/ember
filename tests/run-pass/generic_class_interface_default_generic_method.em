#$ test: run-pass
#$ rules: IFC-1, TYP-16
#$ profiles: debug, development, release
#$ stdout: 42
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(em_7Payload7projectG99f65fd6187c945dd81bcbe71ff877a7)

interface Project:
    [T]
    fn project(value: T) -> T:
        return value

[A]
class Payload implements Project:
    value: A

[A]
class Factory:
    value: A

    fn project_default(payload: Payload[A], value: i32) -> i32:
        return payload.project(value)

    fn round_trip(payload: Payload[A]) -> Payload[A]:
        return payload

fn main():
    payload = Payload[i32](7)
    println(payload.project(42))
    factory = Factory[i32](7)
    println(factory.project_default(payload, 42))
    println(factory.round_trip(payload).project(42))
