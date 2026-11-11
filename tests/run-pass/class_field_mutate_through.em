#$ test: run-pass
#$ rules: CLS-9, CLS-9a, EXC-1, EXC-4, EXC-5
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c-count: contains("ember_object_begin_write") == 1
#$ assert-c-count: contains("ember_object_end_write") == 1

# A field without `let` may be changed through (`[CLS-9a]`: after `init`, nothing is changed through a
# `let` field, so this holder's field is not `let`; the same program with `let` is
# `CLS-9a/reject_a_let_class_field_is_not_changed_through.em`).
# ODR-085: `holder.bump_counter()` holds the holder while other code runs, so
# its write stays; `bump` is quiet and nothing holds a `Counter`, so the inner
# call's check is removed (`[EXC-3]`, `[EXC-19]`).
class Counter:
    value: i32

    fn bump():
        this.value = this.value + 1

class Holder:
    counter: Counter

    fn init(value: i32):
        this.counter = Counter(value)

    fn bump_counter():
        this.counter.bump()

fn main():
    holder = Holder(41)
    holder.bump_counter()
    println(holder.counter.value)
