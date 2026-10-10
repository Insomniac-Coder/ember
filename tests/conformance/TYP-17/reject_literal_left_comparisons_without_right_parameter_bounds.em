#$ test: compile-fail
#$ rules: TYP-17, IFC-4
# The bound requirement is symmetric: a literal on the left does not hide
# an unconstrained right operand. Report the missing bound once at the use,
# before a secondary literal/type mismatch can obscure it.

interface Holder:
    type Item

fn direct_equal[T](x: T) -> bool:
    return 0 == x    #$ error[E2040]: Eq

fn direct_unequal[T](x: T) -> bool:
    return 0 != x    #$ error[E2040]: Eq

fn direct_less[T](x: T) -> bool:
    return 0 < x    #$ error[E2040]: Ord

fn direct_less_equal[T](x: T) -> bool:
    return 0 <= x    #$ error[E2040]: Ord

fn direct_greater[T](x: T) -> bool:
    return 0 > x    #$ error[E2040]: Ord

fn direct_greater_equal[T](x: T) -> bool:
    return 0 >= x    #$ error[E2040]: Ord

fn projected_equal[H: Holder](x: H.Item) -> bool:
    return 0 == x    #$ error[E2040]: Eq

fn projected_unequal[H: Holder](x: H.Item) -> bool:
    return 0 != x    #$ error[E2040]: Eq

fn projected_less[H: Holder](x: H.Item) -> bool:
    return 0 < x    #$ error[E2040]: Ord

fn projected_less_equal[H: Holder](x: H.Item) -> bool:
    return 0 <= x    #$ error[E2040]: Ord

fn projected_greater[H: Holder](x: H.Item) -> bool:
    return 0 > x    #$ error[E2040]: Ord

fn projected_greater_equal[H: Holder](x: H.Item) -> bool:
    return 0 >= x    #$ error[E2040]: Ord

fn main():
    pass
