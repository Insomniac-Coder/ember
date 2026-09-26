#$ test: run-fail
#$ rules: EXC-1, EXC-6, EXC-15, EXC-19
#$ profiles: debug
#$ panics: active read of Bag.items began at
# A whole-object access uses the same debug record as a field access.
class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = [1]

    fn replace(mut self):
        self.items = [2]

fn main():
    b = Bag()
    alias = b
    view = b.items.as_span()
    alias.replace()
    println(view[0])
