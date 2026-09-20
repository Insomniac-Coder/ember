#$ test: compile-fail
#$ rules: TYP-19

struct Payload[A] implements Missing:
    value: A

struct Factory[A]:
    fn make(self, payload: Payload[A]) -> i32:
        return 0

fn main():
    factory = Factory[i32]()
    println(factory.make(Payload[i32](7)))
#$ error[E1010]: cannot find interface `Missing`
