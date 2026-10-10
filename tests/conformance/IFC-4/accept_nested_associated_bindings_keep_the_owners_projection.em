#$ test: run-pass
#$ rules: IFC-4, TYP-17, GRM-8c
#$ stdout: Some(9) None
#$ stdout: 2.5
# A binding containing another associated name substitutes that name under
# Option, and each concrete owner keeps its own Atom and Wrapped types.

interface Inner:
    type Item
    fn item(self) -> Item

interface Outer:
    type Atom: Copy
    type Wrapped: Inner[Item = Option[Atom]]
    fn wrapped(self) -> Wrapped

struct Held[T: Copy]:
    value: Option[T]

extend[T: Copy] Held[T] implements Inner:
    type Item = Option[T]
    fn item(self) -> Option[T]:
        return self.value

struct IntOwner:
    value: Option[int]

extend IntOwner implements Outer:
    type Atom = int
    type Wrapped = Held[int]
    fn wrapped(self) -> Held[int]:
        return Held[int](self.value)

struct FloatOwner:
    value: float

extend FloatOwner implements Outer:
    type Atom = float
    type Wrapped = Held[float]
    fn wrapped(self) -> Held[float]:
        return Held[float](Some(self.value))

fn through[O: Outer](o: O) -> Option[O.Atom]:
    return o.wrapped().item()

fn main():
    println(through(IntOwner(Some(9))), through(IntOwner(None)))
    println(through(FloatOwner(2.5)).unwrap())
