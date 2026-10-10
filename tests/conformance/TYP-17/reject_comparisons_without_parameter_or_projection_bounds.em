#$ test: compile-fail
#$ rules: TYP-17, IFC-4
# Both a written parameter and a projected one need the bound providing
# each comparison. No call is needed to diagnose the generic body.

interface Holder:
    type Item

interface EqualHolder:
    type Item: Eq

fn direct_equal[T](a: T, b: T) -> bool:
    return a == b    #$ error[E2040]: Eq

fn direct_unequal[T](a: T, b: T) -> bool:
    return a != b    #$ error[E2040]: Eq

fn direct_less[T](a: T, b: T) -> bool:
    return a < b    #$ error[E2040]: Ord

fn direct_less_equal[T](a: T, b: T) -> bool:
    return a <= b    #$ error[E2040]: Ord

fn direct_greater[T](a: T, b: T) -> bool:
    return a > b    #$ error[E2040]: Ord

fn direct_greater_equal[T](a: T, b: T) -> bool:
    return a >= b    #$ error[E2040]: Ord

fn projected_equal[H: Holder](a: H.Item, b: H.Item) -> bool:
    return a == b    #$ error[E2040]: Eq

fn projected_unequal[H: Holder](a: H.Item, b: H.Item) -> bool:
    return a != b    #$ error[E2040]: Eq

fn projected_less[H: Holder](a: H.Item, b: H.Item) -> bool:
    return a < b    #$ error[E2040]: Ord

fn projected_less_equal[H: Holder](a: H.Item, b: H.Item) -> bool:
    return a <= b    #$ error[E2040]: Ord

fn projected_greater[H: Holder](a: H.Item, b: H.Item) -> bool:
    return a > b    #$ error[E2040]: Ord

fn projected_greater_equal[H: Holder](a: H.Item, b: H.Item) -> bool:
    return a >= b    #$ error[E2040]: Ord

fn equality_is_not_ordering[T: Eq](a: T, b: T) -> bool:
    return a < b    #$ error[E2040]: Ord

fn projected_equality_is_not_ordering[H: EqualHolder](a: H.Item, b: H.Item) -> bool:
    return a < b    #$ error[E2040]: Ord

fn main():
    pass
