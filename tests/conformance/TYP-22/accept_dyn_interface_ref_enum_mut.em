#$ test: run-pass
#$ rules: TYP-22, IFC-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_4Bump em_vt_dyn_4Bump_for_6Signal)
#$ assert-c: contains(em_vt_dyn_4Bump_for_6Signal_slot0)

interface Bump:
    fn bump() -> i32

enum Signal implements Bump:
    Ready

    fn bump() -> i32:
        return 42

fn bump_it(ref mut value: dyn Bump) -> i32:
    return value.bump()

fn main():
    signal = Signal.Ready
    println(bump_it(ref mut signal))
