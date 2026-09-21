#$ test: run-pass
#$ rules: TYP-22, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_Bump em_vt_dyn_Bump_Signal)
#$ assert-c: contains(em_vt_dyn_Bump_Signal_slot0)

interface Bump:
    fn bump(mut self) -> i32

enum Signal implements Bump:
    Ready

    fn bump(mut self) -> i32:
        return 42

fn bump_it(value: ref mut dyn Bump) -> i32:
    return value.bump()

fn main():
    signal = Signal.Ready
    println(bump_it(ref mut signal))
