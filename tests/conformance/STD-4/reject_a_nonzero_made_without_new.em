#$ test: compile-fail
#$ rules: STD-4, MOD-2
# `[STD-4]` — `NonZero.new` is the only way to make a `NonZero`, which is how
# none holds 0: its field and its memberwise constructor are private to
# `std.core` (`[MOD-2]`).

from std.core import NonZero

fn main():
    _forged = NonZero(0) #$ error[E1052]: `NonZero[i64]`'s memberwise constructor is private to its module
    n = NonZero.new(3).unwrap()
    println(n.value) #$ error[E1052]: `value` is private to `NonZero[i64]`'s module
