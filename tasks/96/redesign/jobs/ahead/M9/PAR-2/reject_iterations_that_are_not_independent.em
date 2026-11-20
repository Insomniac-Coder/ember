# Clause PAR-2.1: "Otherwise `E7010`, whose message gives the lines that make the loop compile"; "a
# task's undeclared `total += x` is a reduction, and a loop body's is `E7010`, whose message gives
# the `reduce=` line"; "a list whose objects may be in it twice (`Array[shares mut Player]`) is
# refused"
#$ test: compile-fail
#$ rules: PAR-2, PAR-3, THR-21
#$ error[E7010]: the parallel loop writes `xs[idx[i]]`, and two rounds may have the same `idx[i]`, so two threads could write one element; run the loop on one thread: remove `@parallel` (line 18)
#$ error[E7010]: reduce=
#$ error[E7010]: `players` holds `shares mut Player` objects, and one object may be in the list twice, so two chunks could change it at once; give each player one owner: write `players: Array[Player] = []` in place of line 30, and `players.push(owns Player(hp=10))` in place of `players.push(shares mut p)` on line 32
# The spec's two messages, with this file's line numbers; the undeclared total claims the part the
# spec gives (`reduce=`). All three are claimed for the file: the spec does not say on which line
# `E7010` stands (Q3).

class Player:
    hp: int = 10

fn scatter(ref mut xs: Array[int], idx: Span[int]):
    @parallel
    for i in 0..idx.len():
        xs[idx[i]] = i

fn total_of(xs: Span[int]) -> int:
    total = 0
    @parallel
    for i in 0..xs.len():
        total += xs[i]
    return total

fn main():
    players: Array[shares mut Player] = []
    p = shares mut Player(hp=10)
    players.push(shares mut p)
    @parallel
    for ref mut q in players:
        q.hp -= 1
    xs = [0, 0]
    scatter(ref mut xs, [0, 1])
    println(total_of([1, 2]))
