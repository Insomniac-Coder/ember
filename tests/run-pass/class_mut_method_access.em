#$ test: run-pass
#$ rules: CLS-1, CLS-7, EXC-1
#$ profiles: debug, release, shipping
#$ assert-c: contains(ember_object_begin_write)
#$ assert-c: contains(ember_object_end_write)
#$ stdout: 7

class Counter:
    value: i32

    fn bump(mut self, amount: i32):
        self.value = self.value + amount

fn main():
    counter = Counter(2)
    counter.bump(5)
    println(counter.value)
