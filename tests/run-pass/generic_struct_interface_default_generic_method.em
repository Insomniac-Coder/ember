#$ test: run-pass
#$ rules: IFC-1, TYP-16
#$ profiles: debug, development, release
#$ assert-c: contains(em_7Payload7projectG99f65fd6187c945d)

interface Project:
    [T]
    fn project(value: T) -> T:
        return value

[A]
struct Payload implements Project:
    value: A

[A]
struct Factory:
    fn project_default(payload: Payload[A], value: i32) -> i32:
        return payload.project(value)

fn main():
    payload = Payload[i32](7)
    println(payload.project(42))
    factory = Factory[i32]()
    println(factory.project_default(payload, 42))
#$ stdout: 42
#$ 42
