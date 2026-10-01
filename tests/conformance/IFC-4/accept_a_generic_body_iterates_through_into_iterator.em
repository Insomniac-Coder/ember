#$ test: run-pass
#$ rules: IFC-4, STD-19
#$ stdout: Some(1) None
# D-407 — `IntoIterator` declares `type Iter: Iterator[Item = Item]`; the
# bound wrote a binding, and a bound with one was dropped, so in a generic
# body `u.into_iter().next()` found no `next` (`E2040`, "`U.Iter` has no
# method `next`"). The bound is kept, and its binding says `U.Iter`'s `Item`
# is `U.Item`.

fn first[U: IntoIterator](owned u: U) -> Option[U.Item]:
    it = u.into_iter()
    return it.next()

fn main():
    xs: Array[int] = [1, 2]
    empty: Array[int] = []
    println(first(xs), first(empty))
