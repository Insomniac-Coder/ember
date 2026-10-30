#$ test: run-fail
#$ rules: TYP-8
#$ profiles: debug, development, release
#$ stdout: 4
#$ stdout: 3
#$ stdout: 2
#$ stdout: 1
#$ stdout: 0
#$ panics: integer overflow in `+`
#$ stderr: run_fail_a_recursive_sum_run_as_a_loop_panics_where_it_did.em:20:12: integer overflow in `+`
# `[TYP-8]` — `grow` runs as a loop with a running total in an optimised build
# (`ember_analysis::accumulate`), and still panics where the function as written does: after the
# same calls, which print what they did, at the written `+`, with its message. `grow(2)` is
# 2^62 + 2^62, one more than the largest `int`.

fn grow(n: int) -> int:
    println(n)
    if n < 2:
        return 4611686018427387904
    return grow(n - 1) + grow(n - 2)

fn main():
    println(grow(4))
