#$ test: run-pass
#$ rules: TYP-17, STD-12, IFC-2
#$ stdout: a b
#$ stdout: c
#$ stdout: false true
# `[TYP-17]` — a bound may name a generic interface with arguments, and the
# arguments may be earlier parameters, the owner's included: `Q: Keyed[K]` is
# the shape of `[STD-12]`'s `AsKey`. Inside the body `q.make_key()` is what the
# bound provides; at a call the bound is checked with the arguments put in.

interface Keyed[K]:
    fn make_key() -> K

extend StrView implements Keyed[String]:
    fn make_key() -> String:
        return String.from(this)

extend String implements Keyed[String]:
    fn make_key() -> String:
        return this.clone()

[Q: needs Keyed[String]]
fn conv_concrete(q: Q) -> String:
    return q.make_key()

[K, Q: needs Keyed[K]]
fn conv(q: Q) -> K:
    return q.make_key()

[K]
struct T2:
    keys: Array[K] = []

    [Q: needs Keyed[K]]
    fn has(q: Q) -> bool:
        k = q.make_key()
        return k in this.keys

fn main():
    println(conv_concrete("a"), conv_concrete(String.from("b")))
    s: String = conv[String, StrView]("c")
    println(s)
    t = T2[String]()
    t.keys.push(owns "b")
    println(t.has("a"), t.has(String.from("b")))
