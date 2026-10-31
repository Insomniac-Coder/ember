#$ test: compile-fail
#$ rules: IFC-4, TYP-17
# D-443 — a default whose `where` clause a type misses is not that type's:
# `Words`, whose `Item` is `String`, has no `wrap` (`where Item: Copy`).

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
    words = Words(w = "a".to_string())
    v = words.wrap()   #$ error[E2072]: `Words` has no method named `wrap`
