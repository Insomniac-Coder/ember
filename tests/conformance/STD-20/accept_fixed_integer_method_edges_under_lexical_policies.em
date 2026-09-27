#$ test: run-pass
#$ rules: STD-20, TYP-8, TYP-28
#$ profiles: debug, release, shipping
#$ stdout: 0 0 -1 4
#$ stdout: 127 -128

@overflow(wrap)
fn wrapped() -> void:
    println(i8.MIN.rem_trunc(-1i8), i128.MIN.rem_trunc(-1i128),
        (-7i8).rem_trunc(2i8), 250u8.wrapping_add(10u8))

@overflow(saturate)
fn saturated() -> void:
    println(127i8.saturating_add(1i8), 127i8.wrapping_add(1i8))

fn main():
    wrapped()
    saturated()
