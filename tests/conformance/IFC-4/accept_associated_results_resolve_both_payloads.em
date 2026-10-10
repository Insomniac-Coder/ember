#$ test: run-pass
#$ rules: IFC-4, TYP-17, ERR-1
#$ stdout: 7 11 13 17
# Result payloads resolve the receiver's associated type in either position,
# including when nested inside another constructed type.

interface Source:
    type Item
    fn success(self) -> Result[Item, int]
    fn failure(self) -> Result[int, Item]
    fn nested(self) -> Option[Result[Item, int]]

struct Numbers:
    n: int

extend Numbers implements Source:
    type Item = int
    fn success(self) -> Result[int, int]:
        return Ok(self.n)
    fn failure(self) -> Result[int, int]:
        return Err(self.n)
    fn nested(self) -> Option[Result[int, int]]:
        return Some(Ok(self.n))

fn success[S: Source](s: S) -> Result[S.Item, int]:
    return s.success()

fn failure[S: Source](s: S) -> Result[int, S.Item]:
    return s.failure()

fn nested[S: Source](s: S) -> Option[Result[S.Item, int]]:
    return s.nested()

fn main():
    a = success(Numbers(7)).unwrap()
    b = failure(Numbers(11)).err().unwrap()
    c = nested(Numbers(13)).unwrap().unwrap()
    d = Numbers(17).success().unwrap()
    println(a, b, c, d)
