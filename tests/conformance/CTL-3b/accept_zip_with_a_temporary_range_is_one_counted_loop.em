#$ test: run-pass
#$ rules: CTL-3b, STD-19, FN-2
#$ profiles: debug, release
#$ stdout: 0 5
#$ stdout: 1 6
#$ stdout: 1 6
#$ stdout: 0 5
#$ stdout: 30 2
#$ stdout: 20 1
#$ stdout: 10 0
#$ stdout: 18446744073709551614 18446744073709551614
#$ stdout: 18446744073709551613 18446744073709551613
#$ stdout: 10 1
#$ stdout: 20 2
#$ stdout: 30 3
#$ stdout: Some(1)
#$ assert-c-count: contains("Zip_std_core_RangeIter_i64_std_core_RangeIter_i64_next") == 0
#$ assert-c-count: contains("Zip_std_core_RangeIter_u64_std_core_RangeIter_u64_next") == 0
# `[CTL-3b]` — `zip` with a range's iterator made in place (`(5..7).iter()`)
# is one counted loop, forwards and backwards. `zip` takes its argument
# borrowed (`[FN-2]`), so the iterator arrives as a borrow of a temporary,
# which the fused loop reads through (D-515); it ran std's `Zip` before. So a
# range too long for an `int` to count zips backwards without asking for its
# length. A named iterator stays the caller's: std's `Zip` copies it, it is
# not moved, and its next item afterwards is the one before the loop.

fn main():
    xs: Array[int] = [10, 20, 30]
    for a, b in (0 .. 2).iter().zip((5 .. 7).iter()):
        println(a, b)
    for a, b in (0 .. 2).iter().zip((5 .. 7).iter()).rev():
        println(a, b)
    for x, i in xs.iter().zip((0 .. 3).iter()).rev():
        println(x, i)
    n = 0
    for a, b in (0 as u64 .. u64.MAX).iter().zip((0 as u64 .. u64.MAX).iter()).rev():
        println(a, b)
        n += 1
        if n == 2:
            break
    it = (0 .. 5).iter()
    it.next()
    for x, i in xs.iter().zip(it):
        println(x, i)
    println(it.next())
