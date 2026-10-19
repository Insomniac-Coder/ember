#$ test: run-pass
#$ rules: CLS-1, EXC-1
#$ profiles: debug, development, release
#$ stdout: 7

class Slot:
    value: i32

fn main():
    slot = Slot(1)
    slot.value = 7
    println(slot.value)
