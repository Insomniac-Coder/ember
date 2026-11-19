#$ test: run-pass
#$ rules: IFC-3
#$ stdout: true 7
# `[IFC-3]` — `Self`, the interface's type parameters and its associated types stand for the
# implementing type and what the `implements` clause and the `type` lines give them, so `fn eq(other:
# Point)` meets `fn eq(other: Self)`, and the same parameter names are the match.

interface Combine:
    type Out
    fn combine(other: Self) -> Out

struct Point:
    x: int

extend Point implements Combine:
    type Out = int

    fn combine(other: Point) -> int:
        return this.x + other.x

extend Point implements Eq:
    fn eq(other: Point) -> bool:
        return this.x == other.x

fn main():
    println(Point(3) == Point(3), Point(3).combine(Point(4)))
