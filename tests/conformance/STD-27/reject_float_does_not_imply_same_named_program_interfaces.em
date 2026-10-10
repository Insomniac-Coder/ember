#$ test: compile-fail
#$ rules: STD-27, TYP-17, TYP-40, IFC-4
# Float implies standard numeric capabilities, not a program's unrelated
# interfaces whose short names happen to be Eq or Default.

from std.math import Float

interface Eq:
    fn eq_marker(self) -> int

interface Default:
    fn default_marker(self) -> int

interface Holder:
    type Item: Float

fn require_local_eq[U: Eq](x: U):
    pass

fn require_local_default[U: Default](x: U):
    pass

fn direct[T: Float](x: T):
    require_local_eq(x)    #$ error[E2040]: Eq
    require_local_default(x)    #$ error[E2040]: Default

fn projected[H: Holder](x: H.Item):
    require_local_eq(x)    #$ error[E2040]: Eq
    require_local_default(x)    #$ error[E2040]: Default

fn main():
    pass
