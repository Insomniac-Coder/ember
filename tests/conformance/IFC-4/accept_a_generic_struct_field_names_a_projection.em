#$ test: run-pass
#$ rules: IFC-4, TYP-16, STD-19
#$ stdout: Some(1) Some(2) Some(3) None
# D-407 (2) — a generic struct's field may name a projection of its own
# parameter (`it: U.Iter`): a hidden parameter, filled from the arguments
# where the type is made, and the same one in a function's signature and
# body that name `Each[U]`, and in an extension of `Each[U]`. It was
# `E2040`, "`U` has no associated type `Iter`": generic structs are collected
# before interfaces.

struct Each[U: IntoIterator]:
    it: U.Iter

fn each[U: IntoIterator](owned u: U) -> Each[U]:
    return Each(it = u.into_iter())

extend[U: IntoIterator] Each[U]:
    fn next_one(mut self) -> Option[U.Item]:
        return self.it.next()

fn main():
    xs: Array[int] = [1, 2]
    e = each(xs)
    ys: Array[int] = [3]
    f = Each[Array[int]](it = ys.into_iter())
    println(e.next_one(), e.next_one(), f.next_one(), f.next_one())
