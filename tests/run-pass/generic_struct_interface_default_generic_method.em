#$ test: run-pass
#$ rules: IFC-1, TYP-16
#$ profiles: debug, release, shipping
#$ assert-c: contains(em_Payload_i32_project__)

interface Project:
    fn project[T](self, value: T) -> T:
        return value

struct Payload[A] implements Project:
    value: A

struct Factory[A]:
    fn project_default(self, payload: Payload[A], value: i32) -> i32:
        return payload.project(value)

fn main():
    payload = Payload[i32](7)
    println(payload.project(42))
    factory = Factory[i32]()
    println(factory.project_default(payload, 42))
#$ stdout: 42
#$ 42
