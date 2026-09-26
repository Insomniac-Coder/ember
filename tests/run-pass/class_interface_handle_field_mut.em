#$ test: run-pass
#$ rules: OBJ-2, DSP-3, EXC-1, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c-count: contains("ember_itable_lookup") == 1
#$ assert-c-count: contains("ember_object_begin_write") == 1
#$ assert-c-count: contains("ember_object_end_write") == 1
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 2

# A mutating call through a class-held interface field dispatches through its
# one-word stored handle. The call is made on a retained copy of it (one
# retain besides the field's own), since the callee could replace the field
# through another handle and free its own receiver mid-call ([RC-5], ODR-065).
# The whole-object write is taken once, by the dynamic adapter, which knows the
# class ([EXC-15]).
interface Accumulator:
    fn bump(mut self) -> i32

class Counter implements Accumulator:
    value: i32

    fn bump(mut self) -> i32:
        self.value = self.value + 1
        return self.value

class Holder:
    value: Accumulator

fn main():
    counter = Counter(41)
    holder = Holder(counter)
    println(holder.value.bump())
