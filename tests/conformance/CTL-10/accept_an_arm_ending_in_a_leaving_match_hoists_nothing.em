#$ test: run-pass
#$ rules: CTL-10, LNT-1
#$ stdout: 2
#$ stdout: 0
# `[CTL-10]` hoists a name only out of branches that complete. An arm whose
# last statement is a `match` with every arm leaving does not complete, so
# `doubled` is not declared after the outer `match`, and no `L1001` says that
# a hoisted copy of it is never read.

fn twice(o: Option[int]) -> int:
    match o:
        Some(n):
            doubled = n * 2
            match doubled > 0:
                true: return doubled
                false: return 0
        None:
            return 0

fn main():
    println(twice(Some(1)))
    println(twice(None))
