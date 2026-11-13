# Clause BRW-17.4: "A view parameter (...) and what its function may end or move may be one place too;
# that is checked at each call (`[BRW-21]`)." `[BRW-21]`'s example: "With `fn note(s: StrView):`
# whose body is `LOG.clear()` then `println(s)`, `note(LOG[0])` is refused ("`note` clears `LOG`
# (its line 3) and then uses `s` (its line 4), and `LOG[0]` is part of `LOG`; give it a copy:
# `note(copy of LOG[0])`, or in `note` use `s` before `LOG.clear()`")"
#$ test: compile-fail
#$ rules: BRW-17, BRW-21, LT-3
# The spec's message, with this file's line numbers.

static mut LOG: Array[String] = []

fn note(s: StrView):
    LOG.clear()
    println(s)

fn main():
    LOG.push(owns String("a"))
    note(LOG[0])    #$ error[E3031]: `note` clears `LOG` (its line 13) and then uses `s` (its line 14), and `LOG[0]` is part of `LOG`; give it a copy: `note(copy of LOG[0])`, or in `note` use `s` before `LOG.clear()`
