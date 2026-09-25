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
    fn make_key(self) -> K

extend str implements Keyed[String]:
    fn make_key(self) -> String:
        return String.from(self)

extend String implements Keyed[String]:
    fn make_key(self) -> String:
        return self.clone()

fn conv_concrete[Q: Keyed[String]](q: Q) -> String:
    return q.make_key()

fn conv[K, Q: Keyed[K]](q: Q) -> K:
    return q.make_key()

struct T2[K]:
    keys: Array[K] = []

    fn has[Q: Keyed[K]](self, q: Q) -> bool:
        k = q.make_key()
        return k in self.keys

fn main():
    println(conv_concrete("a"), conv_concrete(String.from("b")))
    s: String = conv[String, str]("c")
    println(s)
    t = T2[String]()
    t.keys.push("b")
    println(t.has("a"), t.has(String.from("b")))
