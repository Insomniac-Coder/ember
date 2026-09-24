#$ test: run-pass
#$ rules: LT-1, BCK-1
#$ stdout: 3
#$ 1
#$ 7
#$ 1
# A view held in a local still points where it came from: a `ref` copied from
# a parameter, a span of a borrowed `Array`, a copy of a span parameter, and
# `Array.get`'s own view all return into the caller's data.

struct Person:
    age: int

fn via_local_ref(r: ref Person) -> ref int:
    q: ref Person = r
    return q.age

fn via_local_span(a: Array[int]) -> ref int:
    s = a.as_span()
    return ref s[0]

fn via_span_copy(p: Span[int]) -> ref int:
    s = p
    return ref s[0]

fn first(a: Array[int]) -> Option[ref int]:
    return a.get(0)

fn main():
    p = Person(age=3)
    println(via_local_ref(p))
    xs = [1, 2]
    println(via_local_span(xs))
    ys = [7]
    println(via_span_copy(ys))
    println(first(xs).unwrap())
