#$ test: run-pass
#$ rules: IFC-4, TYP-17, GRM-8c
#$ stdout: Some(9) None
#$ stdout: 2.5
# A binding containing another associated name substitutes that name under
# Option, and each concrete owner keeps its own Atom and Wrapped types.

interface Inner:
    type Item
    fn item() -> Item

interface Outer:
    type Atom: needs Copy
    type Wrapped: needs Inner[Item = Option[Atom]]
    fn wrapped() -> Wrapped

[T: needs Copy]
struct Held:
    value: Option[T]

extend[T: needs Copy] Held[T] implements Inner:
    type Item = Option[T]
    fn item() -> Option[T]:
        return this.value

struct IntOwner:
    value: Option[int]

extend IntOwner implements Outer:
    type Atom = int
    type Wrapped = Held[int]
    fn wrapped() -> Held[int]:
        return Held[int](this.value)

struct FloatOwner:
    value: float

extend FloatOwner implements Outer:
    type Atom = float
    type Wrapped = Held[float]
    fn wrapped() -> Held[float]:
        return Held[float](Some(this.value))

[O: needs Outer]
fn through(o: O) -> Option[O.Atom]:
    return o.wrapped().item()

fn main():
    println(through(IntOwner(Some(9))), through(IntOwner(None)))
    println(through(FloatOwner(2.5)).unwrap())
