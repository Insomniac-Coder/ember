interface Inner:
    type Item
    fn item(self) -> Item

interface Outer:
    type Atom
    type Wrapped: Inner[Item = Option[Atom]]
    fn wrapped(self) -> Wrapped

fn get[O: Outer](o: O) -> Option[O.Atom]:
    return o.wrapped().item()

fn main():
    pass
