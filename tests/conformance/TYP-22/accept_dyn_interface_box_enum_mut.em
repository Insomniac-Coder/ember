#$ test: run-pass
#$ rules: TYP-22, IFC-1, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(em_vt_dyn_Bump_Signal_slot0)

interface Bump:
    fn bump(mut self) -> i32

enum Signal implements Bump:
    Ready

    fn bump(mut self) -> i32:
        return 42

fn main():
    boxed: Box[dyn Bump] = Box(Signal.Ready)
    println(boxed.bump())
