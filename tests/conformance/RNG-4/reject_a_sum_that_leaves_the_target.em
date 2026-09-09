#$ test: compile-fail
#$ rules: RNG-4, RNG-4a
# Integer arithmetic derives an interval the same way, and `[RNG-4a]` bounds
# what may be derived at all: a fact is kept only where the operation
# provably cannot overflow its representation, "in any profile", because
# `[TYP-8]`'s policy differs between `debug` and `release` and `[PRF-1]`
# forbids the emitted checks from depending on that.
#
# Here the sum is in `0 ..= 200`, which `u8` holds and `Small` does not.

type Small = u8 in 0 ..= 100

fn main():
    a: Small = 10
    b: Small = 20
    s = a + b
    t: Small = s              #$ error[E2215]: a `Small` cannot be built from a value this is not known to be in range
    v: u8 = t
    println(v)
