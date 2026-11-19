#$ test: compile-fail
#$ rules: TYP-35, THR-1, CELL-3
## `[TYP-35]` — a phantom parameter counts for `Send` and `Sync` as a field of
## its type would, so a marker can make a type thread-confined without a field
## (D-580: it counted for nothing).

[Tag]
struct Handle:
    index: u32

enum Tagged[Tag]:
    Empty
    Index(u32)

class Struct_field:
    let h: Handle[Cell[int]]

    fn init():
        this.h = Handle[Cell[int]](1)

class Enum_field:
    let t: Tagged[Cell[int]]

    fn init():
        this.t = Tagged[Cell[int]].Empty

[Tag]
open class Base:
    let index: u32

    fn init():
        this.index = 1

fn main():
    _a = thread_shared Struct_field()    #$ error[E7015]: `Struct_field.h` is a `Handle[Cell[i64]]`
    _b = thread_shared Enum_field()    #$ error[E7015]: `Enum_field.t` is a `Tagged[Cell[i64]]`
    _c = thread_shared Base[Cell[int]]()    #$ error[E7015]: several threads may not read a `Base[Cell[i64]]` at once
