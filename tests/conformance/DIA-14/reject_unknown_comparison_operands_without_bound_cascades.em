#$ test: compile-fail
#$ rules: DIA-14, TYP-17, IFC-4
# An unknown operand already failed. The other operand's generic type must
# not add a missing Eq/Ord error, regardless of which side contains the name.

interface Holder:
    type Item

[T]
fn direct_left(x: T) -> bool:
    return absent_left < x    #$ error[E1010]: cannot find `absent_left`

[T]
fn direct_right(x: T) -> bool:
    return x == absent_right    #$ error[E1010]: cannot find `absent_right`

[H: needs Holder]
fn projected_left(x: H.Item) -> bool:
    return absent_projected_left != x    #$ error[E1010]: cannot find `absent_projected_left`

[H: needs Holder]
fn projected_right(x: H.Item) -> bool:
    return x >= absent_projected_right    #$ error[E1010]: cannot find `absent_projected_right`

fn main():
    pass
