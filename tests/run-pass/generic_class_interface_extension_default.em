#$ test: run-pass
#$ rules: TYP-16, IFC-1, TYP-22
#$ profiles: debug, development, release
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_6Answer em_vt_dyn_6Answer_for_6HolderGda5cc12a188c7619b25d920e0657739a)

interface Answer:
    fn answer() -> i32:
        return 42

[T]
class Holder:
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
