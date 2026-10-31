#$ test: run-pass
#$ rules: STD-27, TYP-17, TYP-40, IFC-4
#$ profiles: debug, development, release
#$ stdout: 7 11 7 11 true
# A program can explicitly add its own interfaces to a float. Their bounds
# remain usable beside Float's standard capabilities, with no name confusion.

from std.math import Float

interface Eq:
    fn eq_marker() -> int

interface Default:
    fn default_marker() -> int

extend f32 implements Eq, Default:
    fn eq_marker() -> int:
        return 7

    fn default_marker() -> int:
        return 11

interface Holder:
    type Item: needs (Float, Eq, Default)

struct Small:
    unused: int

extend Small implements Holder:
    type Item = f32

[U: needs Eq]
fn local_eq(x: U) -> int:
    return x.eq_marker()

[U: needs Default]
fn local_default(x: U) -> int:
    return x.default_marker()

[T: needs (Float, Eq)]
fn direct_eq(x: T) -> int:
    return local_eq(x)

[T: needs (Float, Default)]
fn direct_default(x: T) -> int:
    return local_default(x)

[H: needs Holder]
fn projected_eq(holder: H, x: H.Item) -> int:
    return local_eq(x)

[H: needs Holder]
fn projected_default(holder: H, x: H.Item) -> int:
    return local_default(x)

[T: needs Float]
fn standard_equality(a: T, b: T) -> bool:
    return a == b

fn main():
    println(direct_eq(1.0f32), direct_default(2.0f32),
            projected_eq(Small(0), 3.0f32), projected_default(Small(0), 4.0f32),
            standard_equality(5.0f32, 5.0f32))
