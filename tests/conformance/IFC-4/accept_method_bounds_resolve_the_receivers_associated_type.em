#$ test: run-pass
#$ rules: IFC-4, TYP-17
#$ stdout: 7 9
# Reading a bound method resolves associated types inside the method's own
# positional bounds, both when the caller fixes Item and when it stays an
# opaque type parameter shared by the receiver and builder bounds.

interface Build[T]:
    fn make(value: T) -> Self

interface Source:
    type Item: Copy
    fn gather[C: Build[Item]](self) -> C

struct Value:
    n: int

extend Value implements Build[int]:
    fn make(value: int) -> Value:
        return Value(value)

struct Numbers:
    n: int

extend Numbers implements Source:
    type Item = int
    fn gather[C: Build[int]](self) -> C:
        return C.make(self.n)

fn fixed[S: Source[Item = int], C: Build[int]](s: S) -> C:
    return s.gather[C]()

fn open_item[T: Copy, S: Source[Item = T], C: Build[T]](s: S) -> C:
    return s.gather[C]()

fn main():
    a = fixed[Numbers, Value](Numbers(7))
    b = open_item[int, Numbers, Value](Numbers(9))
    println(a.n, b.n)
