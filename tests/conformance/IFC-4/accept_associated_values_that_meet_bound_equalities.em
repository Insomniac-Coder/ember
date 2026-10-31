#$ test: run-pass
#$ rules: IFC-4, TYP-17
#$ profiles: debug, development, release
#$ stdout: 7 Some(11) 1
# The matching controls exercise explicit, nested and default associated
# values through concrete owners and through generic calls.

interface Element:
    type Item
    fn get() -> Item

interface Holder:
    type Value: needs Element[Item = int]
    fn value() -> Value

interface NestedHolder:
    type Value: needs Element[Item = Option[int]]
    fn value() -> Value

@derive(Copy)
struct Scalar:
    number: int

extend Scalar implements Element:
    type Item = int
    fn get() -> int:
        return this.number

@derive(Copy)
struct Nested:
    number: int

extend Nested implements Element:
    type Item = Option[int]
    fn get() -> Option[int]:
        return Some(this.number)

struct Owner:
    number: int

extend Owner implements Holder:
    type Value = Scalar
    fn value() -> Scalar:
        return Scalar(this.number)

struct NestedOwner:
    number: int

extend NestedOwner implements NestedHolder:
    type Value = Nested
    fn value() -> Nested:
        return Nested(this.number)

interface DefaultHolder:
    type Value: needs Element[Item = int] = Scalar

struct DefaultOwner:
    unused: int

extend DefaultOwner implements DefaultHolder:
    pass

[H: needs Holder]
fn scalar(holder: H) -> int:
    return holder.value().get()

[H: needs NestedHolder]
fn nested(holder: H) -> Option[int]:
    return holder.value().get()

[H: needs DefaultHolder]
fn defaulted(holder: H) -> int:
    return 1

fn main():
    println(scalar(Owner(7)), nested(NestedOwner(11)), defaulted(DefaultOwner(0)))
