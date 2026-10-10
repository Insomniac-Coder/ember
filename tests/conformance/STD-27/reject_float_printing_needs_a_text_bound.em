#$ test: compile-fail
#$ rules: STD-27, TYP-17, IFC-4, STD-9
# A missing printing bound is E2040, not an unimplemented-feature diagnostic.

from std.math import Float

interface Holder:
    type Item: Float

fn direct[T: Float](x: T):
    println(x)    #$ error[E2040]

fn projected[H: Holder](x: H.Item):
    println(x)    #$ error[E2040]

fn main():
    pass
