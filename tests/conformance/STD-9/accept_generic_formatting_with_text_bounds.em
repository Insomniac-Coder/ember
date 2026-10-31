#$ test: run-pass
#$ rules: STD-9, LEX-19, TXT-9, TYP-17, IFC-4, TYP-39
#$ profiles: debug, development, release
#$ stdout: [   hello] 7
#$ [2, 3] Some(4) (5, 6) Wrapper(item=7)
#$ [8, 9] Some([10, 11])
#$ [      12] 13
# Direct and projected text bounds support f-strings, conversion helpers and
# aggregate printing. Container elements carry Debug, as the contract requires.

interface Holder:
    type Item: needs Debug

struct Integers:
    unused: int

extend Integers implements Holder:
    type Item = int

[T]
struct Wrapper:
    item: T

[T: needs Display]
fn direct(x: T) -> String:
    return f"[{x:>8}]"

[T: needs Debug]
fn text(x: T) -> String:
    return x.to_string()

[T: needs Debug]
fn aggregates(xs: Array[T], choice: Option[T], pair: (int, T), wrapper: Wrapper[T]):
    println(xs, choice, pair, wrapper)

[H: needs Holder]
fn projected(holder: H, xs: Array[H.Item], nested: Option[Array[H.Item]]):
    println(xs, nested)

[H: needs Holder]
fn projected_fstring(holder: H, x: H.Item) -> String:
    return f"[{x:>8}]"

[H: needs Holder]
fn projected_text(holder: H, x: H.Item) -> String:
    return x.to_string()

fn main():
    println(direct("hello"), text(7))
    aggregates([2, 3], Some(4), (5, 6), Wrapper(7))
    projected(Integers(0), [8, 9], Some([10, 11]))
    println(projected_fstring(Integers(0), 12), projected_text(Integers(0), 13))
