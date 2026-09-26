#$ test: run-pass
#$ rules: CLS-9, CLS-9a, EXC-1, EXC-4, EXC-5
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c-count: contains("ember_object_begin_write") == 2
#$ assert-c-count: contains("ember_object_end_write") == 2

# `let` freezes the field binding, not the object stored by that field.
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
