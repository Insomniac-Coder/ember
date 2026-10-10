#$ test: compile-fail
#$ rules: DIA-14, TYP-17, IFC-4
# An unknown operand already failed. The other operand's generic type must
# not add a missing Eq/Ord error, regardless of which side contains the name.

interface Holder:
    type Item

fn direct_left[T](x: T) -> bool:
    return absent_left < x    #$ error[E1010]: cannot find `absent_left`

fn direct_right[T](x: T) -> bool:
    return x == absent_right    #$ error[E1010]: cannot find `absent_right`

fn projected_left[H: Holder](x: H.Item) -> bool:
    return absent_projected_left != x    #$ error[E1010]: cannot find `absent_projected_left`

fn projected_right[H: Holder](x: H.Item) -> bool:
    return x >= absent_projected_right    #$ error[E1010]: cannot find `absent_projected_right`

fn main():
    pass
