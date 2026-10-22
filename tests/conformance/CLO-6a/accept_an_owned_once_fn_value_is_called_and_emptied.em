#$ test: run-pass
#$ rules: CLO-6a, CLO-6, CLO-3, CLO-10, CLO-11
#$ profiles: debug, development, release
#$ stdout: 5
#$ 4 t
#$ 9
#$ 6 2
#$ 10
#$ 4
#$ 7 8
## `[CLO-6a]` — an owned `once fn` value is callable wherever it is held: a local, a struct's
## field (`[CLO-11]`), a `Box`, an element a collection's own API hands over (`pop_front`, `pop`,
## `take`). The call moves the value out of its place: the local, the field, the box are empty
## after it, and a closure that moves a capture out runs with that capture its own. A `fn` value
## is a `once fn` value too. Captured state too big for the value's three words lives in one
## allocation, freed by the call; a `once fn` never called is dropped with its captures.

from std.collections import Deque

struct Task:
    name: String
    run: once fn() -> int

fn consume(owned s: String) -> int:
    return s.len()

fn call_given(f: once fn() -> int) -> int:
    return f()

fn main():
    s = "hello".to_string()
    f: once fn() -> int = owned fn() => consume(s)
    println(f())
    a = "aaaa".to_string()
    t = Task("t".to_string(), owned fn() => consume(a))
    println(t.run(), t.name)
    boxed: Box[once fn() -> int] = Box(owned fn() => 9)
    println(boxed())
    jobs: Deque[once fn() -> int] = Deque.new()
    for word in ["x", "yy", "zzz"]:
        w = word.to_string()
        jobs.push_back(owned fn() => consume(w))
    total = 0
    while true:
        match jobs.pop_front():
            Some(job):
                total += job()
            None:
                break
    stack: Array[once fn() -> int] = []
    stack.push(fn() => 1)
    stack.push(fn() => 2)
    last = stack.pop().unwrap()
    println(total, last())
    b1 = "1".to_string()
    b2 = "22".to_string()
    b3 = "333".to_string()
    b4 = "4444".to_string()
    wide: once fn() -> int = owned fn() => consume(b1) + consume(b2) + consume(b3) + consume(b4)
    println(wide())
    kept = "kept".to_string()
    many: fn() -> int = owned fn() => kept.len()
    once_of_many: once fn() -> int = many
    println(once_of_many())
    unused = "never".to_string()
    _never: once fn() -> int = owned fn() => consume(unused)
    later: Option[once fn() -> int] = Some(fn() => 7)
    given: once fn() -> int = fn() => 8
    println(later.take().unwrap()(), call_given(given))
