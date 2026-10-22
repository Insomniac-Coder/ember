#$ test: run-pass
#$ rules: TYP-22, IFC-1, BRW-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_11Accumulator em_vt_dyn_11Accumulator_for_7Counter)
#$ assert-c: contains(em_7Counter4bump(&receiver))
#$ assert-c: !contains(ember_retain)

interface Accumulator:
    fn bump(mut self) -> i32

class Counter implements Accumulator:
    value: i32

    fn bump(mut self) -> i32:
        self.value = self.value + 1
        return self.value

fn bump_it(value: ref mut dyn Accumulator) -> i32:
    return value.bump()

fn main():
    counter = Counter(41)
    println(bump_it(ref mut counter))
