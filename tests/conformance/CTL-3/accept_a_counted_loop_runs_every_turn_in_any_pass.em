#$ test: run-pass
#$ rules: CTL-3
#$ profiles: debug, release
#$ stdout: 6 5 5 5
#$ stdout: 100 49 12
#$ stdout: 0 1 5 0
#$ stdout: 55 6
# `[CTL-3]` — a counted loop runs every turn once, in order, whatever shape
# the C takes: for MSVC a small loop with a check or a call is written
# several turns to a pass, then the turns left over one at a time (none, a
# few, or all of them when there are fewer than a pass), counting from below
# zero or up to the largest `int` alike; a loop carrying a running total
# keeps one turn to a pass.

class Tally:
    count: int

    fn init(mut self):
        self.count = 0

    fn bump(mut self):
        self.count += 1

fn total_of(xs: Array[int]) -> int:
    total = 0
    for i in 0..len(xs):
        total += xs[i]
    return total

fn main():
    tallies: Array[Tally] = []
    for k in 0..4:
        tallies.push(Tally())
    # 21 turns: whole passes, then the rest.
    for i in 0..21:
        t = tallies[i % 4]
        t.bump()
    println(tallies[0].count, tallies[1].count, tallies[2].count, tallies[3].count)

    squares: Array[int] = []
    for k in 0..12:
        squares.push(0)
    for i in 0..=10:
        squares[i] = i * i
    println(squares[10], squares[7], len(squares))

    seen: Array[int] = []
    for k in 0..4:
        seen.push(0)
    for i in 3..3:
        seen[0] = seen[0] + 1
    for i in -3..-2:
        seen[1] = seen[1] + 1
    for i in 9223372036854775802..9223372036854775807:
        seen[2] = seen[2] + 1
    for i in 5..2:
        seen[3] = seen[3] + 1
    println(seen[0], seen[1], seen[2], seen[3])

    ones: Array[int] = []
    for k in 1..=10:
        ones.push(k)
    println(total_of(ones), total_of(seen))
