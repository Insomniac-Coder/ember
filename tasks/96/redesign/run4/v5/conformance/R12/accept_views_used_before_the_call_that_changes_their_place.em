#$ test: run-pass
#$ rules: BRW-1
#$ stdout: 0
#$ stdout: 11
## design rules: R12, R8
## R12 (rules.md lines 717-720), R8 (f), (g): a read-only view and a `ref mut` name of `c.n` whose last
## uses come before the call through `bump` do not live across it. The nearest allowed program to
## reject_a_read_only_view_across_a_call_that_changes_what_it_reads and
## reject_a_ref_mut_name_across_a_call_that_changes_its_place.

struct Counter:
    n: int = 0

fn main():
    c = Counter()
    bump = fn(): c.n += 1
    r = ref c.n
    println(r)
    w = ref mut c.n
    w += 10
    bump()
    println(c.n)
