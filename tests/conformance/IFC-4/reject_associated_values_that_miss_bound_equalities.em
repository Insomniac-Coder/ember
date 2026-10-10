#$ test: compile-fail
#$ rules: IFC-4, TYP-17
# Meeting the nominal interface is insufficient: associated bindings in the
# declared bound apply to explicit values, nested types, and defaults alike.

interface Element:
    type Item

interface Holder:
    type Value: Element[Item = int]

interface NestedHolder:
    type Value: Element[Item = Option[int]]

struct Wrong:
    unused: int

extend Wrong implements Element:
    type Item = str

struct WrongNested:
    unused: int

extend WrongNested implements Element:
    type Item = Option[str]

struct Owner:
    unused: int

extend Owner implements Holder:    #$ error[E2040]
    type Value = Wrong

struct NestedOwner:
    unused: int

extend NestedOwner implements NestedHolder:    #$ error[E2040]
    type Value = WrongNested

interface DefaultHolder:
    type Value: Element[Item = int] = Wrong

struct DefaultOwner:
    unused: int

extend DefaultOwner implements DefaultHolder:    #$ error[E2040]
    pass

fn main():
    pass
