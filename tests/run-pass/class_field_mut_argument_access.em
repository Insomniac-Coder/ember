#$ test: run-pass
#$ rules: CLS-1, EXC-1, FN-2a
#$ profiles: debug, release, shipping
#$ assert-c: !contains(ember_access_begin_write)
#$ assert-c: !contains(ember_access_end_write)
#$ stdout: 2

class Slot:
    value: i32

fn increment(mut value: i32):
    value = value + 1

fn main():
    slot = Slot(1)
    increment(slot.value)
    println(slot.value)
