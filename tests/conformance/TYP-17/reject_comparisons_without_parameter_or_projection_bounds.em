#$ test: compile-fail
#$ rules: TYP-17, IFC-4
# Both a written parameter and a projected one need the bound providing
# each comparison. No call is needed to diagnose the generic body.

interface Holder:
    type Item

interface EqualHolder:
    type Item: needs Eq

[T]
fn direct_equal(a: T, b: T) -> bool:
    return a == b    #$ error[E2040]: Eq

[T]
fn direct_unequal(a: T, b: T) -> bool:
    return a != b    #$ error[E2040]: Eq

[T]
fn direct_less(a: T, b: T) -> bool:
    return a < b    #$ error[E2040]: Ord

[T]
fn direct_less_equal(a: T, b: T) -> bool:
    return a <= b    #$ error[E2040]: Ord

[T]
fn direct_greater(a: T, b: T) -> bool:
    return a > b    #$ error[E2040]: Ord

[T]
fn direct_greater_equal(a: T, b: T) -> bool:
    return a >= b    #$ error[E2040]: Ord

[H: needs Holder]
fn projected_equal(a: H.Item, b: H.Item) -> bool:
    return a == b    #$ error[E2040]: Eq

[H: needs Holder]
fn projected_unequal(a: H.Item, b: H.Item) -> bool:
    return a != b    #$ error[E2040]: Eq

[H: needs Holder]
fn projected_less(a: H.Item, b: H.Item) -> bool:
    return a < b    #$ error[E2040]: Ord

[H: needs Holder]
fn projected_less_equal(a: H.Item, b: H.Item) -> bool:
    return a <= b    #$ error[E2040]: Ord

[H: needs Holder]
fn projected_greater(a: H.Item, b: H.Item) -> bool:
    return a > b    #$ error[E2040]: Ord

[H: needs Holder]
fn projected_greater_equal(a: H.Item, b: H.Item) -> bool:
    return a >= b    #$ error[E2040]: Ord

[T: needs Eq]
fn equality_is_not_ordering(a: T, b: T) -> bool:
    return a < b    #$ error[E2040]: Ord

[H: needs EqualHolder]
fn projected_equality_is_not_ordering(a: H.Item, b: H.Item) -> bool:
    return a < b    #$ error[E2040]: Ord

fn main():
    pass
