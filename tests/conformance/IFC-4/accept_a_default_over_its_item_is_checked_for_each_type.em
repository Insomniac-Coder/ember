#$ test: run-pass
#$ rules: IFC-4, TYP-17
#$ stdout: 3
#$ stdout: a
# D-441, D-443 — a default whose signature names a bounded type over `Item`
# (`W[Self, Item]`, `W`'s `T: Copy`) under `where Item: Copy`: the bound is
# met for each implementing type (it was asked of `Self.Item` itself), and a
# type that misses the `where` clause (`Words`, whose `Item` is `String`) does
# not have the default, and still implements the interface.

interface Src:
    type Item
    fn get() -> Option[Item]

    fn wrap() owns this -> W[Self, Item] where Item: needs Copy:
        return W(this, None)

[S, T: needs Copy]
struct W:
    s: S
    last: Option[T]

struct Ints:
    n: int

extend Ints implements Src:
    type Item = int

    fn get() -> Option[int]:
        return Some(this.n)

struct Words:
    w: String

extend Words implements Src:
    type Item = String

    fn get() -> Option[String]:
        return Some(this.w.clone())

fn main():
    w = Ints(n = 3).wrap()
    println(w.s.n)
    words = Words(w = "a".to_string())
    println(words.w)
