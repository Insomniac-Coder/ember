#$ test: run-pass
#$ rules: IFC-4, TYP-17
#$ profiles: debug, release, shipping
#$ stdout: 7 Some(11) 1
# The matching controls exercise explicit, nested and default associated
# values through concrete owners and through generic calls.

interface Element:
    type Item
    fn get(self) -> Item

interface Holder:
    type Value: Element[Item = int]
    fn value(self) -> Value

interface NestedHolder:
    type Value: Element[Item = Option[int]]
    fn value(self) -> Value

@derive(Copy)
struct Scalar:
    number: int

extend Scalar implements Element:
    type Item = int
    fn get(self) -> int:
        return self.number

@derive(Copy)
struct Nested:
    number: int

extend Nested implements Element:
    type Item = Option[int]
    fn get(self) -> Option[int]:
        return Some(self.number)

struct Owner:
    number: int

extend Owner implements Holder:
    type Value = Scalar
    fn value(self) -> Scalar:
        return Scalar(self.number)

struct NestedOwner:
    number: int

extend NestedOwner implements NestedHolder:
    type Value = Nested
    fn value(self) -> Nested:
        return Nested(self.number)

interface DefaultHolder:
    type Value: Element[Item = int] = Scalar

struct DefaultOwner:
    unused: int

extend DefaultOwner implements DefaultHolder:
    pass

fn scalar[H: Holder](holder: H) -> int:
    return holder.value().get()

fn nested[H: NestedHolder](holder: H) -> Option[int]:
    return holder.value().get()

fn defaulted[H: DefaultHolder](holder: H) -> int:
    return 1

fn main():
    println(scalar(Owner(7)), nested(NestedOwner(11)), defaulted(DefaultOwner(0)))
