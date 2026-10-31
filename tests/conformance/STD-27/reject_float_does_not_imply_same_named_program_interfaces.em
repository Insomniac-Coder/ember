#$ test: compile-fail
#$ rules: STD-27, TYP-17, TYP-40, IFC-4
# Float implies standard numeric capabilities, not a program's unrelated
# interfaces whose short names happen to be Eq or Default.

from std.math import Float

interface Eq:
    fn eq_marker() -> int

interface Default:
    fn default_marker() -> int

interface Holder:
    type Item: needs Float

[U: needs Eq]
fn require_local_eq(x: U):
    pass

[U: needs Default]
fn require_local_default(x: U):
    pass

[T: needs Float]
fn direct(x: T):
    require_local_eq(x)    #$ error[E2040]: Eq
    require_local_default(x)    #$ error[E2040]: Default

[H: needs Holder]
fn projected(x: H.Item):
    require_local_eq(x)    #$ error[E2040]: Eq
    require_local_default(x)    #$ error[E2040]: Default

fn main():
    pass
