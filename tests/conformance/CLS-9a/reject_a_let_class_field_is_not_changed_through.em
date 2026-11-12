#$ test: compile-fail
#$ rules: CLS-9a, CLS-9
# `[CLS-9a]` — after `init`, nothing is changed through a `let` field, for every type: a class-object
# field is never pointed at another object nor its object changed through it (`E2104`). (This
# program was a run-pass test, `let` freezing only the binding, which was the rule before 0.9.20.)

class Counter:
    value: i32

    fn bump():
        this.value = this.value + 1

class Holder:
    let counter: Counter

    fn init(value: i32):
        this.counter = Counter(value)

    fn bump_counter():
        this.counter.bump()    #$ error[E2104]: `counter` is `let`: it is set in `init` and never changed after; to change it, write the field without `let`: `counter: Counter`

fn main():
    holder = Holder(41)
    holder.bump_counter()
    println(holder.counter.value)
