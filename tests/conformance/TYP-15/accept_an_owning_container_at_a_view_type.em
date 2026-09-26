#$ test: run-pass
#$ rules: TYP-15, STD-11, TYP-15a
#$ stdout: 1 ['a'] {'k': 2} {'s'}
# `[TYP-15]` (ODR-069, SP-013) — an owning container may be written at a view
# type: `Array[str]()`, `Map[str, int]()`, `Set[str]()` and `Span[str]` are
# types like any other, and what enters them must be `static`, checked at each
# store. This replaces ERR-044's refusal "at the type" (2026-09-09), which the
# owner's adoption of SP-013 (2026-09-26) supersedes: one storage rule for
# every owning container. `[TYP-15a]`'s arena containers still hold views
# bounded by their arena.

fn main():
    xs: Array[str] = Array[str]()
    xs.push("a")
    view: Span[str] = xs.as_span()
    m = Map[str, int]()
    m["k"] = 2
    s = Set[str]()
    s.add("s")
    println(view.len(), xs, m, s)
