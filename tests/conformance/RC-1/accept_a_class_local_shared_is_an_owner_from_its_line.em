#$ test: run-pass
#$ rules: RC-1, TYP-45
#$ stdout: 2 2 2
#$ stdout: 2 2
#$ stdout: end 2
#$ stdout: 1
#$ stdout: end 1
#$ stdout: end 2
#$ assert-c-count: contains("ember_retain") == 3
# `[RC-1]` — a class local (or an `owns` parameter) a line of its function shares is one of the owners from
# the line that declares it: its type is `shares mut Counter` throughout, so `c` keeps what it could do, and
# that line takes over the one owner with no count. Each `shares` line adds one; a class local no line
# shares (`alone`) is not counted.

class Counter:
    n: int = 0

    fn bump():
        this.n += 1

    fn drop():
        println(f"end {this.n}")

fn give(owns c: Counter):
    d = shares mut c
    d.bump()
    c.bump()
    println(c.n, d.n)

fn main():
    c = Counter()
    d = shares mut c
    d.bump()
    c.bump()
    r = shares c
    println(c.n, d.n, r.n)
    give(owns Counter())
    alone = Counter()
    alone.bump()
    println(alone.n)
