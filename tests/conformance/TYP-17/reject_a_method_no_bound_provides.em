#$ test: compile-fail
#$ rules: TYP-17
## "There is no duck typing; a missing bound is `E2040` with a suggested bound."

from std.core import Ord

fn call[T](x: T) -> T:
    return x.cmp(x)      #$ error[E2040]: `T` has no method `cmp`
fn main(): pass
