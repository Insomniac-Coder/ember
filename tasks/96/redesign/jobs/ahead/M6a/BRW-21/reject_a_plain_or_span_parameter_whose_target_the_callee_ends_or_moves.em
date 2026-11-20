# Clause BRW-21.2: "for a plain or `Span` parameter, that what it views is not ended or moved"; "an
# argument that points into what the callee may end or move before its last use of that parameter
# is `E3031`"
#$ test: compile-fail
#$ rules: BRW-21, BCK-8, EFF-25
# The first message is the spec's, with this file's line numbers. `add_then_read` may move the
# numbers `xs` views (a push may move the list) and then reads `xs`: code only.

static mut LOG: Array[String] = []
static mut NUMS: Array[int] = []

fn note(s: StrView):
    LOG.clear()
    println(s)

fn add_then_read(xs: Span[int]):
    NUMS.push(owns 1)
    println(xs[0])

fn main():
    LOG.push(owns String("a"))
    NUMS.push(owns 5)
    note(LOG[0])    #$ error[E3031]: `note` clears `LOG` (its line 13) and then uses `s` (its line 14), and `LOG[0]` is part of `LOG`; give it a copy: `note(copy of LOG[0])`, or in `note` use `s` before `LOG.clear()`
    add_then_read(NUMS)    #$ error[E3031]
