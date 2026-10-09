interface Inner:
    type Item
    fn item(self) -> Item

interface Outer:
    type Atom = int
    type Wrapped: Inner[Item = Option[int]]
    fn wrapped(self) -> Wrapped

fn get[O: Outer](o: O) -> Option[int]:
    return o.wrapped().item()

fn main():
    pass
