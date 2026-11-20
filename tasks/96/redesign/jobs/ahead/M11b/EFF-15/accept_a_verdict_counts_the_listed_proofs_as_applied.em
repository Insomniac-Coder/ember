# Clause EFF-15.1: "The proofs that count are exactly those this document specifies — `[OPT-2]`,
# `[RNG-4]`, `[SIMD-7]`, `[DSJ-11]` and the parent-link proof of Part VIII (...) — and a verdict is
# computed **as if every one of them were applied wherever its conditions hold**, whether or not the
# emitted code keeps the check a proof would remove."
#$ test: run-pass
#$ rules: EFF-15, DSJ-11, EFF-12, DSJ-2
#$ profiles: debug, development, release
#$ stdout: 3
# `left` and `right` are the two results of one `split_at`, which `[DSJ-11]` knows apart with no
# compare, so the `assert_disjoint` of them carries no `RuntimeCheck(Aliasing)` in the verdict, in
# every profile, and `@no_alias_checks` holds. Whether such an assert is still an `Aliasing` check
# is Q1; this file follows reading (A).

from std.mem import assert_disjoint

@no_alias_checks
fn add_halves(ref mut xs: MutSpan[int]) -> int:
    left, right = ref mut xs.split_at(1)
    assert_disjoint(left, right)
    left[0] += right[0]
    return left[0]

fn main():
    xs = [1, 2]
    println(add_halves(ref mut xs.as_mut_span()))
