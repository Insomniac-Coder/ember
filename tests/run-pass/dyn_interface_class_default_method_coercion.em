#$ test: run-pass
#$ rules: TYP-22, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_Answer em_vt_dyn_Answer_Pixel)
#$ assert-c: contains(slot0)

interface Answer:
    fn answer(self) -> i32:
        return 42

class Pixel implements Answer:
    pass

fn answer_it(value: ref dyn Answer) -> i32:
    return value.answer()

fn main():
    pixel = Pixel()
    println(answer_it(ref pixel))
