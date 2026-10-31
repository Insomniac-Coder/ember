#$ test: run-pass
#$ rules: IFC-4, TYP-16, TYP-17
#$ stdout: 7 8 9 10 11 12 13 14 15
# Associated-type substitution is structural: user enums/classes and builtin
# wrappers must be rebuilt just as structs, arrays, tuples and Option are.

enum Packet[T]:
    Value(T)

[T]
class Handle:
    value: T

[T]
struct Record:
    value: T

interface Source:
    type Item: needs Copy
    fn packet() -> Packet[Item]
    fn handle() -> Handle[Item]
    fn boxed() -> Box[Item]
    fn cell() -> Cell[Item]
    fn record() -> Record[Item]
    fn fixed() -> [Item; 2]
    fn tuple() -> (Item, Option[Item])
    fn array() -> Array[Item]

struct Numbers:
    n: int

extend Numbers implements Source:
    type Item = int
    fn packet() -> Packet[int]:
        return Packet[int].Value(this.n)
    fn handle() -> Handle[int]:
        return Handle[int](this.n)
    fn boxed() -> Box[int]:
        return Box(this.n)
    fn cell() -> Cell[int]:
        return Cell(this.n)
    fn record() -> Record[int]:
        return Record[int](this.n)
    fn fixed() -> [int; 2]:
        return [this.n; 2]
    fn tuple() -> (int, Option[int]):
        return (this.n, Some(this.n + 1))
    fn array() -> Array[int]:
        return [this.n]

[S: needs Source]
fn packet(s: S) -> Packet[S.Item]:
    return s.packet()

[S: needs Source]
fn handle(s: S) -> Handle[S.Item]:
    return s.handle()

[S: needs Source]
fn boxed(s: S) -> Box[S.Item]:
    return s.boxed()

[S: needs Source]
fn cell(s: S) -> Cell[S.Item]:
    return s.cell()

[S: needs Source]
fn record(s: S) -> Record[S.Item]:
    return s.record()

[S: needs Source]
fn fixed(s: S) -> [S.Item; 2]:
    return s.fixed()

[S: needs Source]
fn tuple(s: S) -> (S.Item, Option[S.Item]):
    return s.tuple()

[S: needs Source]
fn array(s: S) -> Array[S.Item]:
    return s.array()

fn main():
    p = packet(Numbers(7))
    match p:
        Packet.Value(n):
            print(n, end = " ")
    h = handle(Numbers(8))
    b = boxed(Numbers(9))
    c = cell(Numbers(10))
    r = record(Numbers(11))
    f = fixed(Numbers(12))
    t = tuple(Numbers(13))
    a = array(Numbers(15))
    println(h.value, b + 0, c.get(), r.value, f[1], t.0, t.1.unwrap(), a[0])
