interface Build[T]:
    fn make() -> Self

interface Source:
    type Item
    fn gather[C: Build[int]](self) -> C

fn through[S: Source[Item = int], C: Build[int]](s: S) -> C:
    return s.gather[C]()

fn main():
    pass
