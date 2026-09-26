#$ test: run-pass
#$ rules: CLS-1, EXC-1, FN-2a
#$ profiles: debug, release, shipping
#$ assert-c: !contains(ember_object_begin_write)
#$ assert-c: !contains(ember_field_begin_write)
#$ stdout: 2
# `[EXC-17]` — a `Copy` field has no access word: passing it to a `mut` parameter
# checks nothing, as an instantaneous write does not.

class Slot:
    value: i32

fn increment(mut value: i32):
    value = value + 1

fn main():
    slot = Slot(1)
    increment(slot.value)
    println(slot.value)
