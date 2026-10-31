#$ test: run-pass
#$ rules: CLS-1, CLS-7, EXC-1, EXC-15
#$ profiles: debug, development, release
#$ assert-c: contains(ember_object_begin_write)
#$ assert-c: contains(ember_object_end_write)
#$ stdout: 7
# `[EXC-15]` — `bump` runs other code (the printing) while it holds the
# object, so its caller takes the whole-object write and keeps it for the call.

class Counter:
    value: i32

    fn bump(amount: i32):
        this.value = this.value + amount
        println(this.value)

fn main():
    counter = Counter(2)
    counter.bump(5)
