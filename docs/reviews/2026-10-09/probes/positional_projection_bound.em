# Open follow-up: positional bound arguments are read before S.Item is declared.
interface Source:
    type Item
interface Build[T]:
    fn make(value: T) -> Self
fn open_item[S: Source, C: Build[S.Item]](s: S):
    pass
fn main():
    pass
