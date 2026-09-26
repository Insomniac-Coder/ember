#$ test: run-pass
#$ rules: EXC-15, EXC-19, OBJ-2, TYP-22
#$ profiles: debug, release, shipping
#$ stdout: [1, 2, 3]
# `[EXC-15]` — a `mut self` call writes every field of the object once. Through
# an interface-typed handle or a `dyn` box the caller does not know the class,
# so the dynamic adapter, which does, takes the access, and the caller takes
# none: a class with a non-`Copy` field is not locked twice (which would panic).

interface Log:
    fn add(mut self, x: int)

class Record implements Log:
    items: Array[int]

    fn init(mut self):
        self.items = []

    fn add(mut self, x: int):
        self.items.push(x)

class Holder:
    log: Log

fn main():
    record = Record()
    holder = Holder(record)
    holder.log.add(1)
    alias = record
    boxed: Box[dyn Log] = Box(alias)
    boxed.add(2)
    record.add(3)
    println(record.items)
