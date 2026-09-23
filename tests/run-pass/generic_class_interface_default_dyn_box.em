#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, DRP-6, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_Answer em_vt_dyn_Answer_Pixel_i64)

interface Answer:
    fn answer(self) -> i32:
        return 42

class Pixel[T] implements Answer:
    value: T

fn answer_it(value: ref dyn Answer) -> i32:
    return value.answer()

fn main():
    pixel = Pixel(0)
    println(answer_it(ref pixel))
    boxed: Box[dyn Answer] = Box(Pixel(0))
    println(boxed.answer())
