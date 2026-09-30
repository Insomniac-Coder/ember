#$ test: run-pass
#$ rules: RNG-4, RNG-10
# `[RNG-4]` tracks "a known range for numeric expressions ... arithmetic on
# known ranges", and `[RNG-10]`(d) makes "a value whose `[RNG-4]` range is
# contained in the target's" one of the ways a range value arises in Safe
# code — with no check, because every value the expression can take is
# already a value of the target.
#
# The fact doing the work here is not the arithmetic: it is that `l` is a
# `Level` and therefore in `0 ..= 10`, which `[RNG-9]` makes an invariant and
# `[RNG-4]` may assume. So `l * 2` is in `0 ..= 20`, which `Score` contains.
#
# Until this existed, (d) admitted a constant and nothing else, and this
# program was `E2215` (D-018). A float's fact comes only from a comparison
# (`[RNG-4a]`, D-392): `RNG-4a/reject_a_float_fact_not_from_a_true_comparison`.

type Level = int in 0 ..= 10
type Score = int in 0 ..= 20

fn main():
    l: Level = 5
    doubled = l * 2
    back: Score = doubled
    v: int = back
    println(v)
#$ stdout: 10
