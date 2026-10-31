#$ test: compile-fail
#$ rules: TYP-17, TYP-21, TYP-40, IFC-4
# An unrelated interface's operator-shaped methods remain ordinary methods.
# Check uncalled bodies too: a future scalar instantiation cannot supply the
# nominal operator bound missing from this declaration.

interface Lookalike:
    fn add(rhs: Self) -> Self
    fn neg() -> Self
    fn pow(rhs: Self) -> Self
    fn add_assign(rhs: Self)

interface Holder:
    type Item: needs Lookalike

[T: needs Lookalike]
fn direct_add(a: T, b: T) -> T:
    return a + b    #$ error[E2040]: Add

[H: needs Holder]
fn projected_add(a: H.Item, b: H.Item) -> H.Item:
    return a + b    #$ error[E2040]: Add

[T: needs Lookalike]
fn direct_negate(a: T) -> T:
    return -a    #$ error[E2040]: Neg

[H: needs Holder]
fn projected_negate(a: H.Item) -> H.Item:
    return -a    #$ error[E2040]: Neg

[T: needs Lookalike]
fn direct_power(a: T, b: T) -> T:
    return a ** b    #$ error[E2040]: Pow

[H: needs Holder]
fn projected_power(a: H.Item, b: H.Item) -> H.Item:
    return a ** b    #$ error[E2040]: Pow

[T: needs Lookalike]
fn direct_assign(ref mut a: T, b: T):
    a += b    #$ error[E2040]: Add

[H: needs Holder]
fn projected_assign(ref mut a: H.Item, b: H.Item):
    a += b    #$ error[E2040]: Add

fn main():
    pass
