#$ test: run-pass
#$ rules: CLS-1, EXC-1, FN-2a, EXP-1
#$ profiles: debug, release, shipping
#$ assert-c: contains(ember_access_begin_write)
#$ assert-c: contains(ember_access_end_write)
#$ stdout: 2

class Slot:
    value: i32

fn increment(mut value: i32):
    value = value + 1

fn next_index(mut value: usize) -> usize:
    value = value + 1
    return value - 1

fn main():
    slot = Slot(1)
    items: Array[Slot] = Array[Slot]()
    items.push(slot)
    state: usize = 0
    increment(items[next_index(state)].value)
    println(items[0].value)
