#$ test: compile-fail
#$ rules: GRM-38, GRM-15
## `[GRM-38]` — a generator expression's later `for` clause runs once for each item of the clauses
## before it: it cannot take a collection over, and it pairs each item before it with its own, so
## those items are copied, and must be `Copy` (iterating a collection borrowed gives references).

fn main():
    words = [String.from("ab"), String.from("cd")]
    letters = (c for w in owned words for c in w.chars())  #$ error[E2040]: a later `for` clause pairs each of these items with its own, and a `String` is not `Copy`
    xs = [1, 2]
    ys = [3]
    pairs = (x + y for x in xs for y in owned ys)  #$ error[E0109]: a later `for` clause of a generator expression cannot take its collection over
    println(letters.count(), pairs.count())
