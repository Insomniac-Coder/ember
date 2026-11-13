# Clause BRW-8.1: "A plain or `ref mut` parameter is passed by address: the callee reads, or
# changes, the caller's place (`[FN-1]`, ODR-024)."
#$ test: run-pass
#$ rules: BRW-8, FN-1
#$ stdout: 5
#$ stdout: 11
# `Score` is not `Copy`, so a plain `s` is the caller's place itself: a number the module
# variable's own name writes across the call shows through `s` (`[FN-1]`). A `ref mut` parameter
# changes the caller's place.

struct Score:
    points: int
    bonus: int

static mut BEST: Score = Score(points=1, bonus=0)

fn report(s: Score):
    BEST.points = 5
    println(s.points)

fn bump(ref mut s: Score):
    s.points += 10

fn main():
    report(BEST)
    mine = Score(points=1, bonus=0)
    bump(ref mut mine)
    println(mine.points)
