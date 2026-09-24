#$ test: run-pass
#$ rules: TYP-5, TYP-14
#$ stdout: ann
#$ 5
#$ 3
#$ 2
#$ 5
#$ 3
# `[TYP-5]` rule 7 — a place of type `T` becomes a `ref T` wherever one is
# expected: an argument, an initialiser, a return, a view struct's field, and
# a generic `ref T` parameter, whose `T` is inferred through the borrow.

struct Person:
    name: String
    age: int

@view
struct Pair:
    a: ref int
    b: ref int

fn name_of(p: ref Person) -> str:
    return p.name

fn age_of(p: ref Person) -> ref int:
    return p.age

fn same[T: Copy](x: ref T) -> T:
    return x

fn main():
    p = Person(name="ann", age=3)
    println(name_of(p))
    y = 5
    r: ref int = y
    println(r)
    println(age_of(p))
    ages = [1, 2]
    println(same(ages[1]))
    println(same(y))
    pair = Pair(a=y, b=p.age)
    println(pair.b)
