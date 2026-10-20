#$ test: run-pass
#$ rules: TYP-41
#$ stdout: 2 3
#$ stdout: 20 20
## `[TYP-41]` (D-773, D-774) — `Pair[int, 2].make()` names the type with a const argument where an
## expression stands; inside a generic body, `View[N](…)` and `View(…)` name the same `View[N]` as
## the signature does, whichever way `N` was read.

struct Pair[T, const N: int]:
    items: [T; N]

    fn make() -> Pair[int, N]:
        return Pair[int, N]([N; N])

    fn view(self) -> View[T, N]:
        return View[T, N](ref self)

    fn inferred(self) -> View[T, N]:
        return View(ref self)

@view
struct View[T, const N: int]:
    owner: ref Pair[T, N]

fn main():
    p = Pair[int, 2].make()
    q = Pair[int, 3 * 1].make()
    println(p.items[1], q.items[2])
    r = Pair[int, 2]([10, 20])
    println(r.view().owner.items[1], r.inferred().owner.items[1])
