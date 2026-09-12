#$ test: run-pass
#$ rules: CELL-10, FN-1, BRW-1
# The primary B4 repair is structural: keep one owner and pass it through a
# `mut` parameter. This case proves that the diagnostic's first suggestion is
# an accepted program, not merely plausible prose.

struct Counter:
    pub value: i32

fn increment(mut counter: Counter):
    counter.value = counter.value + 1

fn main():
    counter = Counter(1)
    increment(counter)
    println(counter.value)
#$ stdout: 2
