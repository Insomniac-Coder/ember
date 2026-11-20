# Clause CLS-4.1: "An `open` or `abstract` class that another package may derive from says in its
# header whether any thread may hold its objects, `implements Send`, `implements ThreadShared`, or an
# interface that needs them, a promise every class derived from it keeps (`[THR-8]`, `[THR-9]`)"
#$ test: compile-fail
#$ rules: CLS-4, THR-8, THR-9, TYP-43
# `Script` promises `ThreadShared`; `Door` adds a `shares` owner, which is one thread's, and breaks
# it where it is declared (`[THR-9]`: `E7015`). No text in the spec: code only. Needs M9's thread
# markers.

class Counter:
    n: int = 0

pub open class Script implements ThreadShared:
    name: String

    fn init(owns name: String):
        this.name = name

class Door(Script):    #$ error[E7015]
    log: shares Counter

    fn init(owns name: String, owns log: shares Counter):
        this.log = log
        super.init(name)

fn main():
    c = shares Counter()
    _d = Door(String("d"), c)
