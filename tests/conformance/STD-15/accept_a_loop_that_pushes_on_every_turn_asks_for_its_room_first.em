#$ test: run-pass
#$ rules: STD-15, PHIL-5, PHIL-11
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("vec_reserve_hint(") == 3
#$ stdout:
#$ 10 45
#$ 7 30
#$ 3 9
#$ 3 3
#$ 1000 4500
#$ 3 3
# A counted loop that pushes onto a list on every path through a turn asks
# for the room its turns need before it starts: a hint that never fails
# and changes nothing but the capacity. `squares` pushes once a turn and
# `picks` once on either branch; `odd` may push nothing on a turn (the
# fewest pushes on a path is zero) and `early` can leave the loop early, so
# neither asks. `grid`'s inner loop asks on each turn of the outer one; the
# room grows at least by doubling, so that stays linear. `few` pushes three
# times in all, which the first growth (four slots) holds: no hint.

fn main():
    squares: Array[int] = []
    for i in 0..10:
        squares.push(i)
    total = 0
    for x in squares:
        total += x
    println(f"{len(squares)} {total}")

    picks: Array[int] = []
    for i in 0..7:
        if i % 2 == 0:
            picks.push(i)
        else:
            picks.push(i * 2)
    total = 0
    for x in picks:
        total += x
    println(f"{len(picks)} {total}")

    odd: Array[int] = []
    for i in 0..7:
        if i % 2 == 1:
            odd.push(i)
    total = 0
    for x in odd:
        total += x
    println(f"{len(odd)} {total}")

    early: Array[int] = []
    for i in 0..100:
        if i == 3:
            break
        early.push(i)
    total = 0
    for x in early:
        total += x
    println(f"{len(early)} {total}")

    grid: Array[int] = []
    for row in 0..100:
        for column in 0..10:
            grid.push(column)
    total = 0
    for x in grid:
        total += x
    println(f"{len(grid)} {total}")

    few: Array[int] = []
    for i in 0..3:
        few.push(i)
    total = 0
    for x in few:
        total += x
    println(f"{len(few)} {total}")
