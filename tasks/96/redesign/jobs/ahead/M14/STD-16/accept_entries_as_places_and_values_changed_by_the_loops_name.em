# Clause STD-16.1: "`entry(k)` is a `MapEntry` with `or_insert(v)`, `or_insert_with(f)` (...) and
# `or_default()` (for `V: needs Default`), each returning `ref mut V`, a place (`counts.entry(owns
# k).or_default() += 1`)"; "`for k, ref mut v in m:` names each value as a place the loop may
# change"; "`values_mut()` (...) yielding (...) `ref mut V`"
#$ test: run-pass
#$ rules: STD-16, BRW-18, STD-17
#$ stdout: 2 1
#$ stdout: a
#$ stdout: b
#$ stdout: 4 2
#$ stdout: 5 3
# Words counted through `entry(...).or_default()` as a place, doubled through the loop's own name,
# and one added to each through `values_mut()`.

fn main():
    counts: Map[String, int] = Map()
    for w in owns [String("a"), String("b"), String("a")]:
        counts.entry(owns w).or_default() += 1
    println(counts["a"], counts["b"])
    for k, ref mut v in counts:
        println(k)
        v *= 2
    println(counts["a"], counts["b"])
    for ref mut v in counts.values_mut():
        v += 1
    println(counts["a"], counts["b"])
