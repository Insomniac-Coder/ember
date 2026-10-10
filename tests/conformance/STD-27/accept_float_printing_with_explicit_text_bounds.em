#$ test: run-pass
#$ rules: STD-27, TYP-17, IFC-4, STD-9
#$ stdout: 1.5 2.5
#$ stdout: 3.5 4.5
#$ stdout: 5.5 6.5
# Float supplies numeric operations; explicit Display or Debug bounds supply
# printing. Direct and projected parameters retain these separate guarantees.

from std.math import Float

interface Small:
    type Item: Float + Display

interface Large:
    type Item: Float + Display

struct SmallOwner:
    marker: int

extend SmallOwner implements Small:
    type Item = f32

struct LargeOwner:
    marker: int

extend LargeOwner implements Large:
    type Item = f64

fn direct[T: Float + Display](a: T, b: T):
    println(a, b)

fn projected[S: Small, L: Large](a: S.Item, b: L.Item):
    println(a, b)

fn debug_text[T: Float + Debug](a: T, b: T):
    println(a, b)

fn main():
    direct(1.5f32, 2.5f32)
    projected[SmallOwner, LargeOwner](3.5f32, 4.5f64)
    debug_text(5.5f64, 6.5f64)
