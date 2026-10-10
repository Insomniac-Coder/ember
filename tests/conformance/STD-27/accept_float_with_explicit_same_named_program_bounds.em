#$ test: run-pass
#$ rules: STD-27, TYP-17, TYP-40, IFC-4
#$ profiles: debug, release, shipping
#$ stdout: 7 11 7 11 true
# A program can explicitly add its own interfaces to a float. Their bounds
# remain usable beside Float's standard capabilities, with no name confusion.

from std.math import Float

interface Eq:
    fn eq_marker(self) -> int

interface Default:
    fn default_marker(self) -> int

extend f32 implements Eq, Default:
    fn eq_marker(self) -> int:
        return 7

    fn default_marker(self) -> int:
        return 11

interface Holder:
    type Item: Float + Eq + Default

struct Small:
    unused: int

extend Small implements Holder:
    type Item = f32

fn local_eq[U: Eq](x: U) -> int:
    return x.eq_marker()

fn local_default[U: Default](x: U) -> int:
    return x.default_marker()

fn direct_eq[T: Float + Eq](x: T) -> int:
    return local_eq(x)

fn direct_default[T: Float + Default](x: T) -> int:
    return local_default(x)

fn projected_eq[H: Holder](holder: H, x: H.Item) -> int:
    return local_eq(x)

fn projected_default[H: Holder](holder: H, x: H.Item) -> int:
    return local_default(x)

fn standard_equality[T: Float](a: T, b: T) -> bool:
    return a == b

fn main():
    println(direct_eq(1.0f32), direct_default(2.0f32),
            projected_eq(Small(0), 3.0f32), projected_default(Small(0), 4.0f32),
            standard_equality(5.0f32, 5.0f32))
