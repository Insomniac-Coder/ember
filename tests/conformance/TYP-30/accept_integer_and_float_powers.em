#$ test: run-pass
#$ rules: TYP-30, TYP-8
#$ stdout:
#$ 1024 1 -9223372036854775808 1
#$ 243 1.4142135623730951 8.0 1.4142135623730951
#$ 100000 49
# `[TYP-30]` — an integer power is exact in the base's type, so it fits
# whenever the true power does (`(-2) ** 63` is `i64`'s minimum). A float base
# takes a float or integer exponent; an untyped base takes a float
# exponent's type (`2 ** 0.5`). `a **= b` is `a = a ** b`.

println(2 ** 10, 3 ** 0, (-2) ** 63, 0 ** 0)
x: u8 = 3
println(x ** 5, 2.0 ** 0.5, 2.0 ** 3, 2 ** 0.5)
n = 5
y = 7
y **= 2
println(10 ** n, y)
