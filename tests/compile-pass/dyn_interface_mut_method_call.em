#$ test: compile-pass
#$ rules: TYP-22, DSP-3, BRW-1
#$ assert-c: contains(struct em_vt_dyn_Accumulator)
#$ assert-c: contains(->slot0)

interface Accumulator:
    fn bump(mut self) -> i32

fn take(x: ref mut dyn Accumulator) -> i32:
    return x.bump()

fn main():
    pass
