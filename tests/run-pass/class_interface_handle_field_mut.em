#$ test: run-pass
#$ rules: OBJ-2, DSP-3, EXC-1, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c-count: contains("ember_itable_lookup") == 1
#$ assert-c-count: contains("ember_access_begin_write") == 1
#$ assert-c-count: contains("ember_access_end_write") == 1
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 1

# A mutating call through a class-held interface field dispatches directly
# through its one-word stored handle, without retaining a temporary handle.
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
