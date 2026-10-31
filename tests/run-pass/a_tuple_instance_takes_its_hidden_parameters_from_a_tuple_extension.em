#$ test: run-pass
#$ stdout: 12
#$ 23
#$ 34
#$ 111
#$ 122
#$ 133
# `Query[Q]` holds a `Q.State`, a hidden parameter of every instance (D-407), and for
# `Q = (Mut[Pos], Vel)` that is what `extend[A: Param, B: Param] (A, B) implements Param` says,
# over `Mut[Pos]`'s and `Vel`'s implementations. Three things made each such type two instances
# ("expected `Query[(Mut[Pos], Vel)]`, found `Query[(Mut[Pos], Vel)]`"): `system`'s signature is
# read before any implementation is collected, so its instance had no `State` (it is now named,
# and filled in once the implementations are known); a tuple takes its extensions only at a first
# method call, so nothing said its `State` (a projection now gives a concrete instance the
# extensions it lacks); and `make[Q]()`'s `Query[Q]` kept `make`'s own `Q.State` in `run`'s body
# (substituting an instance now substitutes its hidden arguments too).
[T]
struct Mut:
    pass

interface Param:
    type Item
    type State
    static fn open() -> State
    static fn get(ref mut s: State, i: int) -> Item

struct Pos:
    x: int

struct Vel:
    x: int

extend Pos implements Param:
    type Item = int
    type State = Array[int]
    static fn open() -> Array[int]:
        return [1, 2, 3]
    static fn get(ref mut s: Array[int], i: int) -> int:
        return s[i]

extend Vel implements Param:
    type Item = int
    type State = Array[int]
    static fn open() -> Array[int]:
        return [10, 20, 30]
    static fn get(ref mut s: Array[int], i: int) -> int:
        return s[i]

extend[T: needs Param] Mut[T] implements Param:
    type Item = T.Item
    type State = T.State
    static fn open() -> T.State:
        return T.open()
    static fn get(ref mut s: T.State, i: int) -> T.Item:
        return T.get(s, i)

extend[A: needs Param, B: needs Param] (A, B) implements Param:
    type Item = (A.Item, B.Item)
    type State = (A.State, B.State)
    static fn open() -> (A.State, B.State):
        return (A.open(), B.open())
    static fn get(ref mut s: (A.State, B.State), i: int) -> (A.Item, B.Item):
        return (A.get(s.0, i), B.get(s.1, i))

[Q: needs Param]
struct Query:
    state: Q.State
    n: int

    fn iter_mut() -> QueryIter[Q]:
        return QueryIter[Q](state=ref mut this.state, n=this.n, at=0)

[Q: needs Param]
struct QueryIter:
    state: ref mut Q.State
    n: int
    at: int

    fn next() -> Option[Q.Item]:
        if this.at >= this.n:
            return None
        i = this.at
        this.at += 1
        return Some(Q.get(this.state, i))

extend[Q: needs Param] QueryIter[Q] implements Iterator:
    type Item = Q.Item

[Q: needs Param]
fn make() -> Query[Q]:
    return Query[Q](state=Q.open(), n=3)

fn system(ref mut q: Query[(Mut[Pos], Vel)], k: int):
    for a, b in q.iter_mut():
        println(a + b + k)

[Q: needs Param, A]
fn run(f: fn(ref mut Query[Q], A), arg: A):
    q = make[Q]()
    f(q, arg)

fn main():
    q = make[(Mut[Pos], Vel)]()
    system(q, 1)
    run(system, 100)
