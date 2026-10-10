#$ test: compile-fail
#$ rules: IFC-4, TYP-17, GRM-8c
# Bounds instantiated for int and str require different Option payloads;
# implementing Element alone does not meet the associated equality.

interface Element:
    type Item

interface Outer[T]:
    type Value: Element[Item = Option[T]]

struct IntElement:
    unused: int

extend IntElement implements Element:
    type Item = Option[int]

struct TextElement:
    unused: int

extend TextElement implements Element:
    type Item = Option[str]

struct WrongIntOwner:
    unused: int

extend WrongIntOwner implements Outer[int]:    #$ error[E2040]
    type Value = TextElement

struct WrongTextOwner:
    unused: int

extend WrongTextOwner implements Outer[str]:    #$ error[E2040]
    type Value = IntElement

fn main():
    pass
