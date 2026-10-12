#$ test: compile-fail
#$ rules: TYP-35, THR-1, CELL-3
## `[TYP-35]` — a phantom parameter counts for `Send` and `Sync` as a field of
## its type would, so a marker can make a type thread-confined without a field
## (D-580: it counted for nothing).

struct Handle[Tag]:
    index: u32

enum Tagged[Tag]:
    Empty
    Index(u32)

@sync
class Shared:
    let h: Handle[Cell[int]]    #$ error[E7001]: field `Shared.h` has type `Handle[Cell[i64]]`, which is not Sync
    let t: Tagged[Cell[int]]    #$ error[E7001]: field `Shared.t` has type `Tagged[Cell[i64]]`, which is not Sync

    fn init(mut self):
        self.h = Handle[Cell[int]](1)
        self.t = Tagged[Cell[int]].Empty

@sync
open class Base[Tag]:
    let index: u32

    fn init(mut self):
        self.index = 1

fn main():
    b = Base[Cell[int]]()    #$ error[E7001]: `Base[Cell[i64]]` has the type argument `Cell[i64]`, which is not Sync
    println(b.index)
