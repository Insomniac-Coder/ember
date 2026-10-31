#$ test: run-pass
#$ rules: IFC-4, GRM-8c
#$ stdout: Some(4) None
# D-407 (4) — a projection of an associated type: `S.Item.Item` is the
# `Item` of `S`'s `Item`. It was `E1010`, "`S` is not a module in scope".

interface Source:
    type Item: needs IntoIterator
    fn get() -> Item

struct Nums:
    xs: Array[int]

extend Nums implements Source:
    type Item = Array[int]
    fn get() -> Array[int]:
        return this.xs.clone()

[S: needs Source]
fn first_of(s: S) -> Option[S.Item.Item]:
    it = s.get().into_iter()
    return it.next()

fn main():
    println(first_of(Nums(xs = [4, 5])), first_of(Nums(xs = [])))
