#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, DRP-6, HEAP-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_6Answer em_vt_dyn_6Answer_for_7PayloadG73498f1dbd18d400)

interface Answer:
    fn answer(self) -> i32:
        return 42

struct Payload[T] implements Answer:
    value: T

fn answer_it(value: ref dyn Answer) -> i32:
    return value.answer()

fn main():
    payload = Payload(0)
    println(answer_it(ref payload))
    boxed: Box[dyn Answer] = Box(Payload(0))
    println(boxed.answer())
