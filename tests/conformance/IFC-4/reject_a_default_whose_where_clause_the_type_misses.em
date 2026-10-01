#$ test: compile-fail
#$ rules: IFC-4, TYP-17
# D-443 — a default whose `where` clause a type misses is not that type's:
# `Words`, whose `Item` is `String`, has no `wrap` (`where Item: Copy`).

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
    words = Words(w = "a".to_string())
    v = words.wrap()   #$ error[E1010]: `Words` has no method named `wrap`
