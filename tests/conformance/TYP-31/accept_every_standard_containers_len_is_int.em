#$ test: run-pass
#$ rules: TYP-31
#$ stdout: 3 5 2 2 3 4 1
## `[TYP-31]` - every standard container's `len()` returns `int`: an `Array`, a `String`, a `Map`, a `Set`, a `Span` and
## a `StrView`; `usize` stays for foreign interfaces.

fn count(n: int) -> int:
    return n

fn main():
    xs = [1, 2, 3]
    s: String = "hello"
    m = {1: 2, 3: 4}
    set = {1, 2}
    span: Span[int] = xs
    view: StrView = "abcd"
    pair = [7]
    println(count(xs.len()), count(s.len()), count(m.len()), count(set.len()), count(span.len()), count(view.len()), count(pair.len()))
