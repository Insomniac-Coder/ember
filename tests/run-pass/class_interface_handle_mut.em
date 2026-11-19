#$ test: run-pass
#$ rules: OBJ-2, DSP-3, EXC-1, IFC-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(ember_object_begin_write)
#$ assert-c: contains(ember_object_end_write)

# A mutable dynamic call through an [OBJ-2] class-interface handle still
# operates on the one-word owning handle.  The concrete method owns the
# long-term object access interval; no fat `dyn` carrier is introduced.
interface Accumulator:
    fn bump() -> i32

class Counter implements Accumulator:
    value: i32

    fn bump() -> i32:
        this.value = this.value + 1
        return this.value

fn main():
    counter: owns dyn Accumulator = owns Counter(41)
    println(counter.bump())
