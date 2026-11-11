#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, DRP-6, HEAP-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_6Answer em_vt_dyn_6Answer_for_5PixelG73498f1dbd18d400e7759d24661b8d8a)

interface Answer:
    fn answer() -> i32:
        return 42

[T]
class Pixel implements Answer:
    value: T

fn answer_it(value: ref dyn Answer) -> i32:
    return value.answer()

fn main():
    pixel = Pixel(0)
    println(answer_it(ref pixel))
    boxed: owns dyn Answer = owns Pixel(0)
    println(boxed.answer())
