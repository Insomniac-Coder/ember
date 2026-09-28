#$ test: run-pass
#$ rules: SIMD-7, SIMD-5, OPT-2
#$ profiles: debug, release, shipping
#$ stdout: 591 594
#$ assert-c: contains(" >> 56ULL)")
# The loop reads the running total as well as adding to it, so the block adds
# into the total itself rather than into an unsigned copy; the blocks are
# still proved safe at run time and every value comes out exact.

# Lent to a function, a list holds values the range facts cannot know
# (`[RNG-4]`), so the checks this test looks at stay.
fn unknown[T](mut xs: Array[T]):
    pass

fn main():
    xs: Array[int] = []
    out: Array[int] = []
    for i in 0..200:
        xs.push(i % 7)
        out.push(0)
    unknown(xs)
    total = 0
    for i in 0..200:
        out[i] = total
        total = total + xs[i]
    println(out[199], total)
