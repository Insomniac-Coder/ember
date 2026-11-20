# Clause EFF-26.2: "A call that runs a function by itself counts as making everything that function
# changes, now or later: (...) a library function that calls the function it is given
# (`scores.for_each(owns fn(s): LOG.push(owns copy of s.name))` changes `LOG`); whatever drives an
# iterator adapter that keeps one (`for x in it:`, `it.sum()`)"
#$ test: compile-fail
#$ rules: EFF-26, BRW-18, CLO-17
# The first message is the spec's, with this file's line numbers; the adapter cases have no text in
# the spec: code only. `thread.spawn`, `scope.spawn` and the job calls are M9's, a foreign
# `callback(retained)` parameter M12's: not here.

static mut LOG: Array[String] = []

struct Score:
    name: String
    points: int

fn tally(s: Score) -> int:
    LOG.push(owns String("seen"))
    return s.points

fn main():
    scores = [Score(String("ann"), 3), Score(String("bob"), 4)]
    LOG.push(owns String("start"))
    for line in LOG:
        println(line)
        scores.for_each(owns fn(s): LOG.push(owns copy of s.name))    #$ error[E3020]: `scores.for_each(...)` (line 26) pushes to `LOG` (through the function it is given) while this loop walks `LOG` (line 24); call it after the loop: move line 26 below the loop
    for line in LOG:
        println(line)
        println(scores.iter().map(owns fn(s) => tally(s)).sum())    #$ error[E3020]
    it = scores.iter().map(owns fn(s) => tally(s))
    for line in LOG:
        println(line)
        for n in it:    #$ error[E3020]
            println(n)
