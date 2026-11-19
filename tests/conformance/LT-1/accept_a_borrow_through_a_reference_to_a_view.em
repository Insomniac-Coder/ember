#$ test: run-pass
#$ rules: LT-1, LT-22, BRW-6, LT-20
#$ stdout: ann bob
# `[BRW-6]`, `[LT-22]` (D-356) — `ref e.a`, where `e` is a local reference to
# a view struct, borrows through `e` what `e` points to, so it carries `e`'s
# own regions (here `pair`'s), not a borrow of the local `e`. The
# result therefore has a field-by-field summary, and compiles; it was `E3065`
# ("field provenance that cannot be inferred soundly"), because `e`'s region slot's path already
# ends in the dereference. (A function taking `ref p: Pair` cannot show it: a view struct is passed
# as a copy, and a reference to the copy would end with the call.)

struct Pair:
    a: StrView
    b: StrView

fn main():
    s = String.from("ann")
    t = String.from("bob")
    pair = Pair(s.as_str(), t.as_str())
    e: ref Pair = ref pair
    (x, y) = (ref e.a, ref e.b)
    println(x, y)
