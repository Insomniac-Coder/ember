#$ test: run-pass
#$ rules: EXC-2, EXC-6, EXC-19
#$ profiles: debug
#$ stdout: 1 2
# Compatible reads both enter and leave the debug access stack.
class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = [1, 2]

fn main():
    b = Bag()
    alias = b
    first = b.items.as_span()
    second = alias.items.as_span()
    println(first[0], second[1])
