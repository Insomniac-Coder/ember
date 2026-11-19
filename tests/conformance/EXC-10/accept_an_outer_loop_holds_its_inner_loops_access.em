#$ test: run-pass
#$ rules: EXC-8, EXC-9, EXC-10, EXC-11, EXC-12
#$ profiles: debug, development, release
#$ stdout: 12
#$ 15
#$ 39
#$ assert-c: !contains("ember_object_begin_write")

# `[EXC-15]` — a method that changes its object is called with no check at run time, in a loop or a nest
# of loops, though another owner of the object exists.

class Counter:
    value: i32
    log: Array[int]

    fn bump():
        this.value = this.value + 1

fn main():
    counter = Counter(0, [])
    alias = shares mut counter
    for r in 0..3:
        for i in 0..4:
            counter.bump()
    println(alias.value)
    for r in 0..3:
        for i in 0..r:
            counter.bump()
    println(alias.value)
    for a in 0..2:
        for b in 0..3:
            for c in 0..4:
                counter.bump()
    println(alias.value)
