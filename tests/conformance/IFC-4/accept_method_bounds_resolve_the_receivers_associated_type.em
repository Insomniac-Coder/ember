#$ test: run-pass
#$ rules: IFC-4, TYP-17
#$ stdout: 7 9
# Reading a bound method resolves associated types inside the method's own
# positional bounds, both when the caller fixes Item and when it stays an
# opaque type parameter shared by the receiver and builder bounds.

interface Build[T]:
    static fn make(value: T) -> Self

interface Source:
    type Item: needs Copy
    [C: needs Build[Item]]
    fn gather() -> C

struct Value:
    n: int

extend Value implements Build[int]:
    static fn make(value: int) -> Value:
        return Value(value)

struct Numbers:
    n: int

extend Numbers implements Source:
    type Item = int
    [C: needs Build[int]]
    fn gather() -> C:
        return C.make(this.n)

[S: needs Source[Item = int], C: needs Build[int]]
fn fixed(s: S) -> C:
    return s.gather[C]()

[T: needs Copy, S: needs Source[Item = T], C: needs Build[T]]
fn open_item(s: S) -> C:
    return s.gather[C]()

fn main():
    a = fixed[Numbers, Value](Numbers(7))
    b = open_item[int, Numbers, Value](Numbers(9))
    println(a.n, b.n)
