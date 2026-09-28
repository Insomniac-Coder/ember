#$ test: run-pass
#$ rules: OPT-2, RNG-4
#$ profiles: debug, release, shipping
#$ stdout: 96
#$ assert-c-count: contains("ember_panic_bounds(") == 2
#$ assert-c-count: contains("ember_ck_add_i64(") == 5
# Both loops are versioned. The range facts bound the inner counter `c` below
# 2 (`[RNG-4]`), so the outer loop's entry test covers `cells[c]` too: its
# unchecked copy holds the inner loop with no check and no copy of its own,
# and its checked copy holds the inner loop versioned. `row + cells[c]`
# appears three times and `total + row` twice (5 adds); the checks stay only
# in checked copies: `sums[r]` once, `cells[c]` once (2).

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
