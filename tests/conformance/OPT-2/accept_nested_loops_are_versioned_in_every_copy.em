#$ test: run-pass
#$ rules: OPT-2
#$ profiles: debug, release, shipping
#$ stdout: 96
#$ assert-c-count: contains("ember_panic_bounds(") == 3
#$ assert-c-count: contains("ember_ck_add_i64(") == 6
# Both loops are versioned. The outer loop's two copies each hold the inner
# loop, which is versioned in each: `row + cells[c]` appears four times and
# `total + row` twice (6 adds). The checks stay only in checked copies: `sums[r]`
# once, `cells[c]` once in each outer copy (3).

fn main():
    sums: Array[int] = [1, 2, 3]
    cells: Array[int] = [10, 20]
    total = 0
    for r in 0..3:
        row = sums[r]
        for c in 0..2:
            row = row + cells[c]
        total = total + row
    println(total)
