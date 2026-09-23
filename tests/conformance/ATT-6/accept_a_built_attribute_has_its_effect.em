#$ test: run-pass
#$ rules: ATT-6, TYP-8
#$ profiles: debug, release, shipping
#$ stdout: -128
# An attribute in the table has exactly its rule's effect: `@overflow(wrap)`
# wraps where the default would panic.

@overflow(wrap)
fn bump(x: i8) -> i8:
    return x + 1

fn main():
    println(bump(127))
