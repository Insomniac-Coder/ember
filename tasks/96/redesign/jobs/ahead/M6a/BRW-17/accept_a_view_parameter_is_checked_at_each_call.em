# Clause BRW-17.4: "A view parameter (a plain, `Span` or `ref mut` one), or `this`, and what its
# function may end or move may be one place too; that is checked at each call (`[BRW-21]`)."
#$ test: run-pass
#$ rules: BRW-17, BRW-21, LT-3, STA-1
#$ stdout: bye
#$ stdout: 0
#$ stdout: a
# `note` clears `LOG` and then uses `s`, which may be part of `LOG`: `note` itself compiles, and a
# call that gives it text `LOG` does not hold is accepted; so is `note_first(LOG[0])`, whose body uses
# `s` before the clear (`[BRW-21]`'s own examples). The refused call is
# reject_a_call_whose_callee_ends_what_its_argument_views.

static mut LOG: Array[String] = []

fn note(s: StrView):
    LOG.clear()
    println(s)

fn note_first(s: StrView):
    println(s)
    LOG.clear()

fn main():
    note("bye")
    println(LOG.len())
    LOG.push(owns String("a"))
    note_first(LOG[0])
