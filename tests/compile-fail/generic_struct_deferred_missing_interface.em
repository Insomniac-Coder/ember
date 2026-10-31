#$ test: compile-fail
#$ rules: TYP-19

[A]
struct Payload implements Missing:
    value: A

[A]
struct Factory:
    fn make(payload: Payload[A]) -> i32:
        return 0

fn main():
    factory = Factory[i32]()
    println(factory.make(Payload[i32](7)))
#$ error[E1010]: cannot find interface `Missing`
