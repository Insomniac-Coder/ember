#$ test: run-pass
#$ rules: IFC-4, TYP-17, ERR-1
#$ stdout: 7 11 13 17
# Result payloads resolve the receiver's associated type in either position,
# including when nested inside another constructed type.

interface Source:
    type Item
    fn success() -> Result[Item, int]
    fn failure() -> Result[int, Item]
    fn nested() -> Option[Result[Item, int]]

struct Numbers:
    n: int

extend Numbers implements Source:
    type Item = int
    fn success() -> Result[int, int]:
        return Ok(this.n)
    fn failure() -> Result[int, int]:
        return Err(this.n)
    fn nested() -> Option[Result[int, int]]:
        return Some(Ok(this.n))

[S: needs Source]
fn success(s: S) -> Result[S.Item, int]:
    return s.success()

[S: needs Source]
fn failure(s: S) -> Result[int, S.Item]:
    return s.failure()

[S: needs Source]
fn nested(s: S) -> Option[Result[S.Item, int]]:
    return s.nested()

fn main():
    a = success(Numbers(7)).unwrap()
    b = failure(Numbers(11)).err().unwrap()
    c = nested(Numbers(13)).unwrap().unwrap()
    d = Numbers(17).success().unwrap()
    println(a, b, c, d)
