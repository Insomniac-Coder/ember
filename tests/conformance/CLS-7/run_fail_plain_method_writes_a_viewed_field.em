#$ test: run-fail
#$ rules: CLS-7, EXC-1, EXC-16, EXC-19
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: write access to Bag.items while a read access to Bag.items is active
# A plain method may write a field, but it must meet the field's live read access.
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
