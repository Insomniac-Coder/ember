#$ test: compile-fail
#$ rules: TYP-17, TYP-21, TYP-40, IFC-4
# An unrelated interface's operator-shaped methods remain ordinary methods.
# Check uncalled bodies too: a future scalar instantiation cannot supply the
# nominal operator bound missing from this declaration.

interface Lookalike:
    fn add(self, rhs: Self) -> Self
    fn neg(self) -> Self
    fn pow(self, rhs: Self) -> Self
    fn add_assign(mut self, rhs: Self)

interface Holder:
    type Item: Lookalike

fn direct_add[T: Lookalike](a: T, b: T) -> T:
    return a + b    #$ error[E2040]: Add

fn projected_add[H: Holder](a: H.Item, b: H.Item) -> H.Item:
    return a + b    #$ error[E2040]: Add

fn direct_negate[T: Lookalike](a: T) -> T:
    return -a    #$ error[E2040]: Neg

fn projected_negate[H: Holder](a: H.Item) -> H.Item:
    return -a    #$ error[E2040]: Neg

fn direct_power[T: Lookalike](a: T, b: T) -> T:
    return a ** b    #$ error[E2040]: Pow

fn projected_power[H: Holder](a: H.Item, b: H.Item) -> H.Item:
    return a ** b    #$ error[E2040]: Pow

fn direct_assign[T: Lookalike](mut a: T, b: T):
    a += b    #$ error[E2040]: Add

fn projected_assign[H: Holder](mut a: H.Item, b: H.Item):
    a += b    #$ error[E2040]: Add

fn main():
    pass
