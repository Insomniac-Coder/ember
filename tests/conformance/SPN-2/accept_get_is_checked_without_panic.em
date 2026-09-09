#$ test: run-pass
#$ rules: SPN-2
## "Indexing a `Span` is bounds-checked; `get(i) -> Option[ref T]` is the
## checked-without-panic form."

fn first(xs: Span[i32]) -> i32:
    match xs.get(0):
        Some(r) => return r
        None => return -1

fn main():
    a: Array[i32] = Array[i32]()
    a.push(7)
    println(first(a))
    empty: Array[i32] = Array[i32]()
    println(first(empty))
#$ stdout: 7
#$ -1
