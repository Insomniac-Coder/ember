#$ test: compile-fail
#$ rules: TYP-17, IFC-4
# The bound requirement is symmetric: a literal on the left does not hide
# an unconstrained right operand. Report the missing bound once at the use,
# before a secondary literal/type mismatch can obscure it.

interface Holder:
    type Item

[T]
fn direct_equal(x: T) -> bool:
    return 0 == x    #$ error[E2040]: Eq

[T]
fn direct_unequal(x: T) -> bool:
    return 0 != x    #$ error[E2040]: Eq

[T]
fn direct_less(x: T) -> bool:
    return 0 < x    #$ error[E2040]: Ord

[T]
fn direct_less_equal(x: T) -> bool:
    return 0 <= x    #$ error[E2040]: Ord

[T]
fn direct_greater(x: T) -> bool:
    return 0 > x    #$ error[E2040]: Ord

[T]
fn direct_greater_equal(x: T) -> bool:
    return 0 >= x    #$ error[E2040]: Ord

[H: needs Holder]
fn projected_equal(x: H.Item) -> bool:
    return 0 == x    #$ error[E2040]: Eq

[H: needs Holder]
fn projected_unequal(x: H.Item) -> bool:
    return 0 != x    #$ error[E2040]: Eq

[H: needs Holder]
fn projected_less(x: H.Item) -> bool:
    return 0 < x    #$ error[E2040]: Ord

[H: needs Holder]
fn projected_less_equal(x: H.Item) -> bool:
    return 0 <= x    #$ error[E2040]: Ord

[H: needs Holder]
fn projected_greater(x: H.Item) -> bool:
    return 0 > x    #$ error[E2040]: Ord

[H: needs Holder]
fn projected_greater_equal(x: H.Item) -> bool:
    return 0 >= x    #$ error[E2040]: Ord

fn main():
    pass
