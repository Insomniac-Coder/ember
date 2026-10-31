#$ test: run-pass
#$ rules: IFC-4, TYP-17
#$ stdout: Some(7) None
# D-407 (3) — a method's signature may name the associated type of its own
# parameter (`U.Elem`), of an interface collected after the one declaring
# the method. It was `E2040`, "`U` has no associated type `Elem`", in the
# interface and in the implementation alike.

interface Walk:
    [U: needs Seq]
    fn first_of(u: U) -> Option[U.Elem]

interface Seq:
    type Elem
    fn first() -> Option[Elem]

struct Nums:
    xs: Array[int]

extend Nums implements Seq:
    type Elem = int
    fn first() -> Option[int]:
        if this.xs.len() == 0:
            return None
        return Some(this.xs[0])

struct W:
    n: int

extend W implements Walk:
    [U: needs Seq]
    fn first_of(u: U) -> Option[U.Elem]:
        return u.first()

fn main():
    w = W(n = 1)
    println(w.first_of(Nums(xs = [7, 8])), w.first_of(Nums(xs = [])))
