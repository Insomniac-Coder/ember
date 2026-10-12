#$ test: compile-fail
#$ rules: CELL-3, THR-1
## `[CELL-3]` — `Cell[T]` may move between threads when `T: Send`, but is never
## shared: the field breaks `Sync` only, and the diagnostic says so.

@sync
class Shared:
    let cell: Cell[int] #$ error[E7001]: field `Shared.cell` has type `Cell[i64]`, which is not Sync

    fn init(mut self):
        self.cell = Cell(1)

fn main():
    pass
