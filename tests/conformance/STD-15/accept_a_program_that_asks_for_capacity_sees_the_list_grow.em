#$ test: run-pass
#$ rules: STD-15, PHIL-5
#$ profiles: debug, release, shipping
#$ assert-c: !contains("vec_reserve_hint(")
#$ stdout:
#$ 5 8
# A list grows by doubling from four. Room asked for before a loop that
# pushes would change what `capacity()` reports, so a program that asks a
# list for its capacity gets no such hint anywhere.

fn main():
    xs: Array[int] = []
    for i in 0..5:
        xs.push(i)
    println(f"{len(xs)} {xs.capacity()}")
