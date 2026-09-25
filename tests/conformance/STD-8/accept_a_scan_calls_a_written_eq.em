#$ test: run-pass
#$ rules: STD-8, STR-5
#$ stdout: true true true Some(1)
#$ true true
# D-294 — `x in xs`, `contains` and `index_of` compare as `==` does: a
# written `eq` replaces the implicit one (`[STR-5]`), here ignoring `tag`,
# and two maps with the same entries in another order are equal. The scan
# used to compare field by field.

struct S:
    x: int
    tag: int

extend S implements Eq:
    fn eq(self, other: S) -> bool:
        return self.x == other.x

fn main():
    xs = [S(1, 0), S(2, 0)]
    println(S(1, 9) == xs[0], S(1, 9) in xs, xs.contains(S(2, 9)), xs.index_of(S(2, 9)))
    a: Map[String, int] = {"x": 1, "y": 2}
    b: Map[String, int] = {"y": 2, "x": 1}
    println(a == b, a in [b])
