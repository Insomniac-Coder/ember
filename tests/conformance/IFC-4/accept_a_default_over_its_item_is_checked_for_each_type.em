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
    fn get(mut self) -> Option[Item]

    fn wrap(owned self) -> W[Self, Item] where Item: Copy:
        return W(self, None)

struct W[S, T: Copy]:
    s: S
    last: Option[T]

struct Ints:
    n: int

extend Ints implements Src:
    type Item = int

    fn get(mut self) -> Option[int]:
        return Some(self.n)

struct Words:
    w: String

extend Words implements Src:
    type Item = String

    fn get(mut self) -> Option[String]:
        return Some(self.w.clone())

fn main():
    w = Ints(n = 3).wrap()
    println(w.s.n)
    words = Words(w = "a".to_string())
    println(words.w)
