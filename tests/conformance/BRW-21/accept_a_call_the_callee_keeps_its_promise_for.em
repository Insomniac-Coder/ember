# Clause BRW-21.2: "`note("bye")` is allowed, and so is `note(LOG[0])` for a `note` that prints
# before it clears"
#$ test: run-pass
#$ rules: BRW-21, BCK-8
#$ stdout: bye
#$ stdout: a
#$ stdout: 5
#$ stdout: 7
# The spec's two allowed calls, and the same for a `Span` parameter: a callee that reads before it
# adds, and a list the callee does not reach by another name.

static mut LOG: Array[String] = []
static mut NUMS: Array[int] = []

fn note(s: StrView):
    LOG.clear()
    println(s)

fn note_first(s: StrView):
    println(s)
    LOG.clear()

fn read_then_add(xs: Span[int]):
    println(xs[0])
    NUMS.push(owns 1)

fn add_then_read(xs: Span[int]):
    NUMS.push(owns 1)
    println(xs[0])

fn main():
    note("bye")
    LOG.push(owns String("a"))
    note_first(LOG[0])
    NUMS.push(owns 5)
    read_then_add(NUMS)
    mine = [7]
    add_then_read(mine)
