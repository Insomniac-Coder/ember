#$ test: run-pass
#$ rules: IFC-4, TYP-16, TYP-17
#$ stdout: 7 8 9 10 11 12 13 14 15
# Associated-type substitution is structural: user enums/classes and builtin
# wrappers must be rebuilt just as structs, arrays, tuples and Option are.

enum Packet[T]:
    Value(T)

class Handle[T]:
    value: T

struct Record[T]:
    value: T

interface Source:
    type Item: Copy
    fn packet(self) -> Packet[Item]
    fn handle(self) -> Handle[Item]
    fn boxed(self) -> Box[Item]
    fn cell(self) -> Cell[Item]
    fn record(self) -> Record[Item]
    fn fixed(self) -> [Item; 2]
    fn tuple(self) -> (Item, Option[Item])
    fn array(self) -> Array[Item]

struct Numbers:
    n: int

extend Numbers implements Source:
    type Item = int
    fn packet(self) -> Packet[int]:
        return Packet[int].Value(self.n)
    fn handle(self) -> Handle[int]:
        return Handle[int](self.n)
    fn boxed(self) -> Box[int]:
        return Box(self.n)
    fn cell(self) -> Cell[int]:
        return Cell(self.n)
    fn record(self) -> Record[int]:
        return Record[int](self.n)
    fn fixed(self) -> [int; 2]:
        return [self.n; 2]
    fn tuple(self) -> (int, Option[int]):
        return (self.n, Some(self.n + 1))
    fn array(self) -> Array[int]:
        return [self.n]

fn packet[S: Source](s: S) -> Packet[S.Item]:
    return s.packet()

fn handle[S: Source](s: S) -> Handle[S.Item]:
    return s.handle()

fn boxed[S: Source](s: S) -> Box[S.Item]:
    return s.boxed()

fn cell[S: Source](s: S) -> Cell[S.Item]:
    return s.cell()

fn record[S: Source](s: S) -> Record[S.Item]:
    return s.record()

fn fixed[S: Source](s: S) -> [S.Item; 2]:
    return s.fixed()

fn tuple[S: Source](s: S) -> (S.Item, Option[S.Item]):
    return s.tuple()

fn array[S: Source](s: S) -> Array[S.Item]:
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
