#$ test: run-pass
#$ rules: CTL-3b, RNG-4, STD-26
#$ profiles: development
#$ stdout: 1668
#$ assert-c: !contains(>> 63LL)
# `[CTL-3b]`, `[RNG-4]` — a stepped range's values lie between its start and its stop, counting
# down as well as up: every `v` here is positive, so `v % 11` needs none of the floor remainder's
# sign fix-ups (it was 1.3x a checked C loop with them).

fn main():
    s = 0
    for v in range(999, 0, -3):
        s += v % 11
    println(s)
