#$ test: run-pass
#$ rules: OBJ-2, DSP-3, EXC-1, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(ember_itable_lookup)
#$ assert-c: contains(ember_object_begin_write)
#$ assert-c: contains(ember_object_end_write)

# A mutable dynamic call through an [OBJ-2] class-interface handle still
# operates on the one-word owning handle.  The concrete method owns the
# long-term object access interval; no fat `dyn` carrier is introduced.
interface Accumulator:
    fn bump(mut self) -> i32

class Counter implements Accumulator:
    value: i32

    fn bump(mut self) -> i32:
        self.value = self.value + 1
        return self.value

fn main():
    counter: Accumulator = Counter(41)
    println(counter.bump())
