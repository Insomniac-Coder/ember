#$ test: compile-fail
#$ rules: STD-27, TYP-17, IFC-4, STD-9
# A missing printing bound is E2040, not an unimplemented-feature diagnostic.

from std.math import Float

interface Holder:
    type Item: needs Float

[T: needs Float]
fn direct(x: T):
    println(x)    #$ error[E2040]

[H: needs Holder]
fn projected(x: H.Item):
    println(x)    #$ error[E2040]

fn main():
    pass
