#$ test: run-pass
#$ rules: TYP-17, IFC-3, IFC-4, TYP-37
#$ profiles: debug, release, shipping
#$ stdout: true false true true false false
#$ true false true true false false
#$ true true true false
# Check every comparison on concrete integer instantiations, Eq inherited
# through Ord, record equality, and IEEE float comparisons through a bound.

interface OrderedHolder:
    type Item: Ord

struct Ints:
    unused: int

extend Ints implements OrderedHolder:
    type Item = int

struct Floats:
    unused: int

extend Floats implements OrderedHolder:
    type Item = f64

struct Record:
    value: int

fn same[T: Eq](a: T, b: T) -> bool:
    return a == b

fn direct[T: Ord](a: T, b: T):
    println(a == a, a != a, a < b, a <= b, a > b, a >= b)

fn projected[H: OrderedHolder](holder: H, a: H.Item, b: H.Item):
    println(a == a, a != a, a < b, a <= b, a > b, a >= b)

fn projected_less[H: OrderedHolder](holder: H, a: H.Item, b: H.Item) -> bool:
    return a < b

fn main():
    direct(2, 3)
    projected(Ints(0), 2, 3)
    nan = 0.0 / 0.0
    println(same(4, 4), same(Record(5), Record(5)),
            projected_less(Floats(0), 1.0, 2.0), projected_less(Floats(0), nan, 2.0))
