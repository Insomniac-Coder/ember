#$ test: run-fail
#$ rules: STD-7
#$ panics: a BitSet holds non-negative integers, not -1
## `[STD-7]` — a `BitSet` holds non-negative integers; inserting a negative one panics.

from std.collections import BitSet

fn main():
    b = BitSet.new()
    _ = b.insert(-1)
