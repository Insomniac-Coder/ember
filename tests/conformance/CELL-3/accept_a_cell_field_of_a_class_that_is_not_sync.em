#$ test: run-pass
#$ rules: CELL-3, CELL-2, THR-1
#$ stdout: 3
## `[CELL-3]` — `Cell[T]` is `!Sync`, which only keeps it out of what threads
## share: a class that is not `@sync` holds one and sets it through a shared
## handle.

class Counter:
    hits: Cell[int]

    fn init(mut self):
        self.hits = Cell(0)

    fn hit(self):
        self.hits.set(self.hits.get() + 1)

fn main():
    counter = Counter()
    alias = counter
    counter.hit()
    alias.hit()
    counter.hit()
    println(alias.hits.get())
