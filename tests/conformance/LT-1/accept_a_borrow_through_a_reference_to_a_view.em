#$ test: run-pass
#$ rules: LT-1, LT-22, BRW-6, LT-20
#$ stdout: ann bob
# `[BRW-6]`, `[LT-22]` (D-356) — `ref e.a`, where `e` is a local reference to
# a view struct, borrows through `e` what `e` points to, so it carries `e`'s
# own regions (here the parameter's), not a borrow of the local `e`. `both`'s
# result therefore has a field-by-field summary, and the call compiles; it
# was `E3065` ("field provenance that cannot be inferred soundly"), because
# `e`'s region slot's path already ends in the dereference.

struct Pair:
    a: str
    b: str

fn both(p: ref Pair) -> (ref str, ref str):
    e: ref Pair = p
    return (ref e.a, ref e.b)

fn main():
    s = String.from("ann")
    t = String.from("bob")
    pair = Pair(s.as_str(), t.as_str())
    (x, y) = both(ref pair)
    println(x, y)
