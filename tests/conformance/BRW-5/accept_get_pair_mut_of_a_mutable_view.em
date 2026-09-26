#$ test: run-pass
#$ rules: BRW-5, SPN-5, FN-1a
#$ stdout: [1, 5, 3, 2]
#$ stdout: [1, 5, 9, 2]
# `[SPN-5]` (ODR-068) — a `MutSpan` has `get_pair_mut` too. Its receiver is
# `mut self` of the view itself, so a view-producing expression
# (`xs.as_mut_span()`) may be the receiver, as `[FN-1a]` lets it be any
# `mut` argument of a view type (D-350).

fn bump_pair(mut view: MutSpan[int]):
    match view.get_pair_mut(3, 1):
        Some((a, b)):
            b = b + a
        None:
            pass

fn main():
    xs = [1, 3, 3, 2]
    bump_pair(xs)
    println(xs)
    match xs.as_mut_span().get_pair_mut(2, 1):
        Some((a, b)):
            a = a + b + 1
        None:
            pass
    println(xs)
