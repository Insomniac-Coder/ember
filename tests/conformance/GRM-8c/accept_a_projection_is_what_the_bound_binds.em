#$ test: run-pass
#$ rules: GRM-8c, IFC-4, TYP-17
#$ stdout: 5
#$ 1
#$ 8
# `[GRM-8c]` (D-675) — a bound's binding says what the associated type is: with `T: Add[Output = T]`,
# `T.Output` in the signature is `T`; with `I: Iterator[Item = int]`, `I.Item` is `int`. The signature
# made a hidden parameter of its own while the body read the binding, so `a + b` (a `T`) did not
# match the declared `T.Output` (E2020), and `x + 1` on an `I.Item` had no `+` (E2040).

[T: needs Add[Output = T]]
fn sum2(a: T, b: T) -> T.Output:
    return a + b

[I: needs Iterator[Item = int]]
fn first(owns it: I) -> I.Item:
    return it.next().unwrap_or(0)

[I: needs Iterator[Item = int]]
fn bump(owns it: I, x: I.Item) -> int:
    return x + 1

fn main():
    println(sum2(2, 3))
    println(first([1, 2].into_iter()))
    println(bump([1].into_iter(), 7))
