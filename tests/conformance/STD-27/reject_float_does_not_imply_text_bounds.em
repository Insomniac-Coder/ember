#$ test: compile-fail
#$ rules: STD-27, TYP-17, IFC-4
# The Float capability list does not provide Display or Debug. Even though its
# concrete implementers have text, a generic body needs an explicit text bound.

from std.math import Float

interface Holder:
    type Item: Float

fn require_display[U: Display](x: U):
    pass

fn require_debug[U: Debug](x: U):
    pass

fn direct[T: Float](x: T):
    require_display(x)    #$ error[E2040]
    require_debug(x)    #$ error[E2040]

fn projected[H: Holder](x: H.Item):
    require_display(x)    #$ error[E2040]
    require_debug(x)    #$ error[E2040]

fn main():
    pass
