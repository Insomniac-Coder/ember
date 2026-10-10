#$ test: run-pass
#$ rules: TYP-17, STD-27, TYP-37, TYP-9, IFC-4
#$ profiles: debug, release, shipping
#$ stdout: false true true true false false
#$ false true false false false false
#$ true false false true false true
#$ false true false false false false
# Float supplies Eq/Ord and numeric literal adoption. Comparing a literal
# with direct or projected Float still uses IEEE comparisons for NaN and zero.

from std.math import Float

interface Holder:
    type Item: Float

struct Doubles:
    unused: int

extend Doubles implements Holder:
    type Item = f64

fn direct[T: Float](x: T):
    println(0 == x, 0 != x, 0 < x, 0 <= x, 0 > x, 0 >= x)

fn projected[H: Holder](holder: H, x: H.Item):
    println(0.0 == x, 0.0 != x, 0.0 < x, 0.0 <= x, 0.0 > x, 0.0 >= x)

fn main():
    direct(1.0f32)
    direct(0.0f32 / 0.0f32)
    projected(Doubles(0), -0.0f64)
    projected(Doubles(0), 0.0f64 / 0.0f64)
