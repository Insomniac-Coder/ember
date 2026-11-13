# Clause BRW-13.6: "A name written `const ref mut` is never given another ref, as `const` on a value
# means it never changes" -- it still writes the place it names.
#$ test: run-pass
#$ rules: BRW-13, TYP-14
#$ stdout: 20 2

fn main():
    a = 1
    b = 2
    r = const ref mut a
    r = 20
    println(a, b)
