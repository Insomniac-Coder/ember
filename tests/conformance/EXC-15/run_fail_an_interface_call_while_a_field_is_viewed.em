#$ test: run-fail
#$ rules: EXC-15, EXC-19, EXC-1, OBJ-2
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: write access to Record.items while a read access to Record.items is active
# `[EXC-15]` — the dynamic adapter's whole-object write checks every field of
# the object; a live view of one through another handle is a conflict.

interface Log:
    fn add(mut self, x: int)

class Record implements Log:
    items: Array[int]

    fn init(mut self):
        self.items = [0]

    fn add(mut self, x: int):
        self.items.push(x)

class Holder:
    log: Log

fn main():
    record = Record()
    holder = Holder(record)
    view = record.items[..1]
    holder.log.add(1)
    println(view)
