#$ test: run-pass
#$ rules: GRM-34, TYP-19
#$ profiles: debug, release
#$ stdout: <5> <x>
#$ <7>
# `[GRM-34]` (ADR-191) — an extension over a type parameter standing alone may have a parameter
# besides, which an interface it implements names: a blanket implementation every type meeting the
# bounds has, its methods generic over that parameter. A call fixes it by its arguments, the type
# wanted of its result, or the one instance of the interface the type implements (inside an
# instance of `shown`, the bound says `Wrap[Labelled]`).

interface Wrap[U]:
    fn wrap(self) -> U

struct Labelled:
    text: String

extend[T: Display, U: From[String]] T implements Wrap[U]:
    fn wrap(self) -> U:
        return U.from(f"<{self}>")

extend Labelled implements From[String]:
    fn from(owned value: String) -> Labelled:
        return Labelled(value)

fn shown[T: Wrap[Labelled]](value: T) -> String:
    return value.wrap().text

fn main():
    println(shown(5), shown("x"))
    l: Labelled = 7.wrap()
    println(l.text)
