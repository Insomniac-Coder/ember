#$ test: run-fail
#$ rules: EXC-1, EXC-6, EXC-19
#$ profiles: debug
#$ panics: active read of Bag.items began at
# The debug panic identifies where the live conflicting field access began.
class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = [1]

    fn replace(self):
        self.items = [2]

fn main():
    b = Bag()
    alias = b
    view = b.items.as_span()
    alias.replace()
    println(view[0])
