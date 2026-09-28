#$ test: run-pass
#$ rules: CTL-3b, CTL-1, SPN-5
#$ profiles: debug, release, shipping
#$ assert-c-count: contains(".index = (") == 0
#$ stdout: 2 20 | 10
# `[CTL-1]`, `[CTL-3b]` — `for x in view` over a `MutSpan` gives each element
# mutably, as `view.iter_mut()` does, in a counted loop; the view is
# reborrowed for the loop, not moved into it, so it is used again after.

fn main():
    xs: Array[int] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    view = xs.as_mut_span()
    for x in view:
        x *= 2
    println(view[0], view[9], "|", xs[4])
