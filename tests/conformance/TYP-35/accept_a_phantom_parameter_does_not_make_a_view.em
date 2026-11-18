#$ test: run-pass
#$ rules: TYP-35
#$ stdout: 7
## `[TYP-35]` - a phantom parameter does not make its type a view: a `Handle[StrView]` outlives any text.

[Tag]
struct Handle:
    index: u32

fn make() -> Handle[StrView]:
    return Handle(7)

fn main():
    h = make()
    println(h.index)
