#$ test: compile-fail
#$ rules: STD-27, TYP-17, IFC-4
# The Float capability list does not provide Display or Debug. Even though its
# concrete implementers have text, a generic body needs an explicit text bound.

from std.math import Float

interface Holder:
    type Item: needs Float

[U: needs Display]
fn require_display(x: U):
    pass

[U: needs Debug]
fn require_debug(x: U):
    pass

[T: needs Float]
fn direct(x: T):
    require_display(x)    #$ error[E2040]
    require_debug(x)    #$ error[E2040]

[H: needs Holder]
fn projected(x: H.Item):
    require_display(x)    #$ error[E2040]
    require_debug(x)    #$ error[E2040]

fn main():
    pass
