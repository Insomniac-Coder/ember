#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, DRP-6, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_Answer em_vt_dyn_Answer_Payload_i32)

interface Answer:
    fn answer(self) -> i32:
        return 42

struct Payload[T] implements Answer:
    value: T

struct Factory[T]:
    value: T

    fn run(self):
        boxed: Box[dyn Answer] = Box(Payload(0))
        println(boxed.answer())

fn main():
    factory = Factory(0)
    factory.run()
