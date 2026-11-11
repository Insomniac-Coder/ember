#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, DRP-6, HEAP-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_6Answer em_vt_dyn_6Answer_for_7PayloadG73498f1dbd18d400e7759d24661b8d8a)

interface Answer:
    fn answer() -> i32:
        return 42

[T]
struct Payload implements Answer:
    value: T

[T]
struct Factory:
    value: T

    fn run():
        boxed: owns dyn Answer = owns Payload(0)
        println(boxed.answer())

fn main():
    factory = Factory(0)
    factory.run()
