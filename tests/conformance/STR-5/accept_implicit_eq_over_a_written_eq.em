#$ test: run-pass
#$ rules: STR-5, TYP-21, TYP-36
#$ stdout: true false true
#$ stdout: true false
#$ stdout: true true false
#$ stdout: true false true true
#$ stdout: true false
#$ stdout: true
# `[STR-5]` — a type has `Eq` when every component has it, and a component
# with a written `eq` has it: `Outer` compares its `S` through `S.eq`, as do
# tuples, `Array`s, fixed arrays, enums and `Option`s holding one. `[TYP-21]`
# — `Eq.eq` gives both `==` and `!=`. `S.eq` ignores `tag` on purpose.

struct S:
    x: int
    tag: int

extend S implements Eq:
    fn eq(self, other: S) -> bool:
        return self.x == other.x

struct Outer:
    s: S
    name: String

enum E:
    One(s: S)
    Two(n: int)
    Nothing

fn main():
    println(S(1, 5) == S(1, 6), S(1, 5) != S(1, 6), S(1, 5) != S(2, 5))
    println(Outer(S(1, 5), "a") == Outer(S(1, 6), "a"), Outer(S(1, 5), "a") == Outer(S(1, 5), "b"))
    println((S(1, 0), 2) == (S(1, 9), 2), [S(1, 0), S(2, 0)] == [S(1, 7), S(2, 7)], [S(1, 0)] == [S(1, 0), S(2, 0)])
    println(E.One(S(3, 0)) == E.One(S(3, 1)), E.One(S(3, 0)) == E.Two(3), E.Nothing == E.Nothing, E.Two(1) != E.Two(2))
    o: Option[S] = Some(S(4, 0))
    n: Option[S] = None
    println(o == Some(S(4, 8)), o == n)
    fixed: [S; 2] = [S(1, 0), S(2, 0)]
    other: [S; 2] = [S(1, 3), S(2, 3)]
    println(fixed == other)
