#$ test: compile-fail
#$ rules: THR-1, THR-9
#$ profiles: debug
#$ error[E7001]: field `Shared.cell` has type `Cell[i64]`, which is not Sync

@sync
class Shared:
    let cell: Cell[int]

    fn init(mut self):
        self.cell = Cell(1)
