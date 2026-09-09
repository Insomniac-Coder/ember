#$ test: compile-fail
#$ rules: SPN-3, BRW-5
# `[SPN-3]` names `s.reborrow()` and `[BRW-5]` the sanctioned ways, but only
# part of the view-method surface exists yet (SPN-API-1 in `docs/BACKLOG.md`).
# What exists refuses by name rather than reading as a typo — pinned here so
# the message cannot silently change, the way `CELL-DEF-1` pins `take`'s.

fn main():
    buf: Array[i32] = Array[i32]()
    buf.push(1)
    buf.push(2)
    left = buf.as_mut_span().split_at(1)   #$ error[E2020]: `MutSpan[i32]` has no method `split_at` in this phase
    println(left.len())
