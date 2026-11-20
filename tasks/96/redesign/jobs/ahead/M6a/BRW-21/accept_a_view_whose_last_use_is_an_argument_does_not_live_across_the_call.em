# Clause BRW-21.1: "A view whose last use is an argument of a call (of a plain, `Span` or `ref mut`
# parameter) does not live across the call: inside it, the parameter makes the promise of `[BRW-1]`,
# and the compiler holds the callee to it"
#$ test: run-pass
#$ rules: BRW-21, BCK-2, BRW-2
#$ stdout: a
#$ stdout: 0
#$ stdout: 3
#$ stdout: 5
# `first` and `view` are last used as arguments, so neither lives across its call: `note_first`
# clears `LOG` after its last use of `s`, and the list is added to after `view`'s last use.

static mut LOG: Array[String] = []

fn note_first(s: StrView):
    println(s)
    LOG.clear()

fn count(xs: Span[int]) -> int:
    return xs.len()

fn grow(ref mut xs: Array[int]):
    xs.push(owns 4)

fn main():
    LOG.push(owns String("a"))
    first = LOG[0]
    note_first(first)
    println(LOG.len())
    nums = [1, 2, 3]
    view = nums.as_span()
    println(count(view))
    nums.push(owns 9)
    grow(ref mut nums)
    println(nums.len())
