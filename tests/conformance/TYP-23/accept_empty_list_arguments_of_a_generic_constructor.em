#$ test: run-pass
#$ rules: TYP-23, TYP-38, STR-1
#$ stdout: 0 0 1
#$ stdout: 0 None
# `[TYP-23]` — once a generic struct's parameters are known, from explicit type
# arguments or from the type the context expects, each constructor argument
# is checked against its field's type, so `[]` and `None` need no annotation.

struct Table[K, V]:
    keys: Array[K]
    values: Array[V]
    live: int

struct Slot[T]:
    items: Array[T]
    first: Option[T]

fn main():
    t = Table[int, String](keys = [], values = [], live = 0)
    u: Table[String, int] = Table([], [], 1)
    println(t.keys.len(), u.values.len(), u.live)
    s = Slot[String](items = [], first = None)
    println(s.items.len(), s.first)
