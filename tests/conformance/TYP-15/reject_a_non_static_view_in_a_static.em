#$ test: compile-fail
#$ rules: TYP-15, LT-3, DIA-7a
# And the half the decision must not weaken. The exception is on the region, so
# an initialiser that is not known to be static-region is refused — a view
# whose region the compiler cannot bound would outlive whatever it borrows.
#
# `E3063`, shape B12 ("a view stored in a place that outlives it"), not `E3060`,
# which is B7 ("a borrowed value does not live long enough"). `[LT-3]` used to
# name `E3060` here; the two are different shapes and the owner reconciled it.

fn pick(s: str) -> str:
    return s

static BAD: str = pick("x")    #$ error[E3063]: `str` is a view, so it may not be stored in a `static`

fn main():
    println(1)
