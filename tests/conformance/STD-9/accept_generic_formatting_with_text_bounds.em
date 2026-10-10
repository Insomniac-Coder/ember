#$ test: run-pass
#$ rules: STD-9, LEX-19, TXT-9, TYP-17, IFC-4, TYP-39
#$ profiles: debug, release, shipping
#$ stdout: [   hello] 7
#$ [2, 3] Some(4) (5, 6) Wrapper(item=7)
#$ [8, 9] Some([10, 11])
#$ [      12] 13
# Direct and projected text bounds support f-strings, conversion helpers and
# aggregate printing. Container elements carry Debug, as the contract requires.

interface Holder:
    type Item: Debug

struct Integers:
    unused: int

extend Integers implements Holder:
    type Item = int

struct Wrapper[T]:
    item: T

fn direct[T: Display](x: T) -> String:
    return f"[{x:>8}]"

fn text[T: Debug](x: T) -> String:
    return x.to_string()

fn aggregates[T: Debug](xs: Array[T], choice: Option[T], pair: (int, T), wrapper: Wrapper[T]):
    println(xs, choice, pair, wrapper)

fn projected[H: Holder](holder: H, xs: Array[H.Item], nested: Option[Array[H.Item]]):
    println(xs, nested)

fn projected_fstring[H: Holder](holder: H, x: H.Item) -> String:
    return f"[{x:>8}]"

fn projected_text[H: Holder](holder: H, x: H.Item) -> String:
    return x.to_string()

fn main():
    println(direct("hello"), text(7))
    aggregates([2, 3], Some(4), (5, 6), Wrapper(7))
    projected(Integers(0), [8, 9], Some([10, 11]))
    println(projected_fstring(Integers(0), 12), projected_text(Integers(0), 13))
