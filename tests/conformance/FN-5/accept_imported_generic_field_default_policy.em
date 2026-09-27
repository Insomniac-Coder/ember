#$ test: run-pass
#$ rules: FN-5, STR-2, TYP-8, MOD-3
#$ profiles: debug, release, shipping
#$ stdout: -128

from support.overflow_field_default import OverflowBox

@overflow(saturate)
fn main():
    box = OverflowBox[int](5)
    println(box.value)
