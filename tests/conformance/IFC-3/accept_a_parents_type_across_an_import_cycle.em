#$ test: run-pass
#$ rules: IFC-3, MOD-4
#$ stdout: 7
# `[IFC-3]`, `[MOD-4]` — modules may import each other; a child interface
# still names its parent's associated type across the cycle (D-458).

from support.ring_child import Ranked
from support.ring_parent import top

fn main():
    println(top(7))
