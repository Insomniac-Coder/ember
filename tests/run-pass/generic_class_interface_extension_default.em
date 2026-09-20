#$ test: run-pass
#$ rules: TYP-16, IFC-1, TYP-22
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_Answer em_vt_dyn_Answer_Holder_bool)

interface Answer:
    fn answer(self) -> i32:
        return 42

class Holder[T]:
    marker: T

extend[T] Holder[T] implements Answer:
    pass

fn answer(value: ref dyn Answer) -> i32:
    return value.answer()

fn main():
    holder = Holder[bool](false)
    println(answer(ref holder))
    boxed: Box[dyn Answer] = Box(Holder[bool](false))
    println(boxed.answer())
