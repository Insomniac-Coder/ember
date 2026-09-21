#$ test: run-pass
#$ rules: TYP-22, IFC-1, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_Answer em_vt_dyn_Answer_Signal)
#$ assert-c: contains(em_vt_dyn_Answer_Signal_slot0)

interface Answer:
    fn answer(self) -> i32:
        return 42

enum Signal implements Answer:
    Ready

fn answer_ref(value: ref dyn Answer) -> i32:
    return value.answer()

fn main():
    signal = Signal.Ready
    println(answer_ref(ref signal))
    boxed: Box[dyn Answer] = Box(Signal.Ready)
    println(boxed.answer())
