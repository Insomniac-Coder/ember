#$ test: run-pass
#$ rules: IFC-3, MOD-4
#$ stdout: 30 10 1
# `[IFC-3]` — a child interface names its parent's associated type wherever
# each is declared and however the modules are imported: here the child's
# module is imported before the parent's (D-458: interfaces were collected
# one module at a time, in an order that is not the order of their parents).

from support.weighed import Weighed, Crate
from support.measured import Measured, touch

fn main():
    c = Crate(weight = 10)
    println(c.heaviest(), c.measure(), touch(1))
