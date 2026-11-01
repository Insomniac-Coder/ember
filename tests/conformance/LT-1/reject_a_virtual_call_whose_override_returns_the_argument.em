#$ test: compile-fail
#$ rules: LT-1, LT-22, BRW-1
#$ not-help: `runtime`
# `[LT-1]` (verdict 151) — `B.first` gives back the list it was handed: `show`, given a `B`, is refused
# where `xs` changes while `r` is used; `keep`, the same code given an `A`, whose `first` gives back the
# object's own list, compiles; `pick`, given an `A` whose class is chosen while running, is refused
# with the kind named (`runtime` is a word only before `dyn`, so no help names it).

open class A:
    let items: Array[int] = [7]

    virtual fn first(other: Array[int]) -> Span[int]:
        return this.items

class B(A):
    override fn first(other: Array[int]) -> Span[int]:
        return other

fn keep(a: A):
    xs = [1]
    r = a.first(xs)
    xs.push(owns 2)
    println(r)

fn show(a: A):
    xs = [1]
    r = a.first(xs)
    xs.push(owns 2)    #$ error[E3021]: `xs` is borrowed here and mutably borrowed elsewhere
    println(r)

fn pick(a: A):
    xs = [1]
    r = a.first(xs)
    xs.push(owns 2)    #$ error[E3021]: `xs` changes while `r` may still look at it: `a` may be a `B`, and which kind it is is only known while running
    println(r)

fn main():
    keep(A())
    show(B())
    a: A = B() if input() == "b" else A()
    pick(a)
