#$ test: run-pass
#$ rules: STD-20
#$ profiles: debug, release, shipping
#$ stdout: 1.0 0.30000000000000004 1e+300 1e+16 1000000000000000.0
#$ 0.0001 1e-05 -0.0 1.5e+16 100.0 2.5
#$ 0.1 inf -inf nan
# A float's `Display` is Python's `repr`: the shortest text that reads back as
# the same value, `.0` on an integral value, and exponent notation outside
# [1e-4, 1e16).

fn main():
    println(1.0, 0.1 + 0.2, 1e300, 1e16, 1e15)
    println(0.0001, 0.00001, -0.0, 1.5e16, 100.0, 2.5)
    tenth: f32 = 0.1
    println(tenth, 1.0 / 0.0, -1.0 / 0.0, 0.0 / 0.0)
