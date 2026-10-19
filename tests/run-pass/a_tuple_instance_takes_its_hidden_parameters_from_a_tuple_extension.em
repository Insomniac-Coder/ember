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
struct Mut[T]:
    pass

interface Param:
    type Item
    type State
    fn open() -> State
    fn get(mut s: State, i: int) -> Item

struct Pos:
    x: int

struct Vel:
    x: int

extend Pos implements Param:
    type Item = int
    type State = Array[int]
    fn open() -> Array[int]:
        return [1, 2, 3]
    fn get(mut s: Array[int], i: int) -> int:
        return s[i]

extend Vel implements Param:
    type Item = int
    type State = Array[int]
    fn open() -> Array[int]:
        return [10, 20, 30]
    fn get(mut s: Array[int], i: int) -> int:
        return s[i]

extend[T: Param] Mut[T] implements Param:
    type Item = T.Item
    type State = T.State
    fn open() -> T.State:
        return T.open()
    fn get(mut s: T.State, i: int) -> T.Item:
        return T.get(s, i)

extend[A: Param, B: Param] (A, B) implements Param:
    type Item = (A.Item, B.Item)
    type State = (A.State, B.State)
    fn open() -> (A.State, B.State):
        return (A.open(), B.open())
    fn get(mut s: (A.State, B.State), i: int) -> (A.Item, B.Item):
        return (A.get(s.0, i), B.get(s.1, i))

struct Query[Q: Param]:
    state: Q.State
    n: int

    fn iter_mut(mut self) -> QueryIter[Q]:
        return QueryIter[Q](state=ref mut self.state, n=self.n, at=0)

struct QueryIter[Q: Param]:
    state: ref mut Q.State
    n: int
    at: int

    fn next(mut self) -> Option[Q.Item]:
        if self.at >= self.n:
            return None
        i = self.at
        self.at += 1
        return Some(Q.get(self.state, i))

extend[Q: Param] QueryIter[Q] implements Iterator:
    type Item = Q.Item

fn make[Q: Param]() -> Query[Q]:
    return Query[Q](state=Q.open(), n=3)

fn system(mut q: Query[(Mut[Pos], Vel)], k: int):
    for a, b in q.iter_mut():
        println(a + b + k)

fn run[Q: Param, A](f: fn(mut Query[Q], A), arg: A):
    q = make[Q]()
    f(q, arg)

fn main():
    q = make[(Mut[Pos], Vel)]()
    system(q, 1)
    run(system, 100)
