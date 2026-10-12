#$ test: compile-fail
#$ rules: CELL-8, THR-1
## `[CELL-8]` — `RefCell[T]` is `!Sync` and, like any type, `Send` when what it
## holds is: the field breaks `Sync` only, and the diagnostic says so. (The
## cell's private record of where a borrow began is not part of what it holds.)

@sync
class Shared:
    let cell: RefCell[int] #$ error[E7001]: field `Shared.cell` has type `RefCell[i64]`, which is not Sync

    fn init(mut self):
        self.cell = RefCell(1)

fn main():
    pass
