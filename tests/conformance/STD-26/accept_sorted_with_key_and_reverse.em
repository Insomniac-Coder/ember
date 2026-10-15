#$ test: run-pass
#$ rules: STD-26
#$ stdout: ['a', 'bb', 'dd', 'ccc'] ['ccc', 'bb', 'dd', 'a'] [3, 2, 1] [1, 9, 16] [1, 4, 7, 10] ['ann', 'cy', 'bob'] [9, 8, 7]
# `[STD-26]` — `sorted(it, key=None, reverse=false)` returns a new sorted array, stable: equal keys
# keep their order, also when reversed. `it` is anything a `for` loop iterates.

struct Person:
    name: String
    age: int

fn main():
    xs = ["bb", "a", "ccc", "dd"]
    a = sorted(xs, key=fn(w) => w.len())
    b = sorted(xs, key=fn(w) => w.len(), reverse=true)
    c = sorted([3, 1, 2], reverse=true)
    d = sorted(x * x for x in [3, -4, 1])
    e = sorted(range(10, 0, -3))
    people = [Person("ann", 30), Person("bob", 25), Person("cy", 30)]
    by_age = sorted(people, key=fn(p) => p.age, reverse=true)
    names = [p.name.clone() for p in by_age]
    fixed: [int; 3] = [9, 7, 8]
    g = sorted(fixed, reverse=true)
    println(a, b, c, d, e, names, g)
