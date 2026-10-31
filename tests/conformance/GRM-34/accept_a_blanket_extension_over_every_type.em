#$ test: run-pass
#$ rules: GRM-34, TYP-19
#$ profiles: debug, development
#$ stdout: <5> <x>
#$ <7>
# `[GRM-34]` (ADR-191) — an extension over a type parameter standing alone may have a parameter
# besides, which an interface it implements names: a blanket implementation every type meeting the
# bounds has, its methods generic over that parameter. A call fixes it by its arguments, the type
# wanted of its result, or the one instance of the interface the type implements (inside an
# instance of `shown`, the bound says `Wrap[Labelled]`).

interface Wrap[U]:
    fn wrap() -> U

struct Labelled:
    text: String

extend[T: needs Display, U: needs From[String]] T implements Wrap[U]:
    fn wrap() -> U:
        return U.from(f"<{this}>")

extend Labelled implements From[String]:
    static fn from(owns value: String) -> Labelled:
        return Labelled(value)

[T: needs Wrap[Labelled]]
fn shown(value: T) -> String:
    return value.wrap().text

fn main():
    println(shown(5), shown("x"))
    l: Labelled = 7.wrap()
    println(l.text)
