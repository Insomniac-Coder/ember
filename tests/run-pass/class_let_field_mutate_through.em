#$ test: run-pass
#$ rules: CLS-9, CLS-9a, EXC-1, EXC-4, EXC-5
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c-count: contains("ember_object_begin_write") == 1
#$ assert-c-count: contains("ember_object_end_write") == 1

# `let` freezes the field binding, not the object stored by that field.
# ODR-085: `holder.bump_counter()` holds the holder while other code runs, so
# its write stays; `bump` is quiet and nothing holds a `Counter`, so the inner
# call's check is removed (`[EXC-3]`, `[EXC-19]`).
class Counter:
    value: i32

    fn bump(mut self):
        self.value = self.value + 1

class Holder:
    let counter: Counter

    fn init(mut self, value: i32):
        self.counter = Counter(value)

    fn bump_counter(mut self):
        self.counter.bump()

fn main():
    holder = Holder(41)
    holder.bump_counter()
    println(holder.counter.value)
