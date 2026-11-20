# Clause DIA-9.1: "A diagnostic never suggests `unsafe` while a fix without it compiles in that
# program, and never suggests `shares` or `shares mut` alone where a fix that keeps one owner
# compiles."
#$ test: compile-fail
#$ rules: DIA-9, BRW-21, BRW-15, DIA-7
#$ not-help: unsafe
#$ not-help: shares
# A call-check refusal (fixed by `copy of` or by reordering the callee) and a change through a plain
# parameter (fixed by `ref mut`): no help mentions `unsafe` or `shares`.

static mut LOG: Array[String] = []

struct Clock:
    frame: int = 0

fn note(s: StrView):
    LOG.clear()
    println(s)

fn tick(s: Clock):
    s.frame += 1    #$ error[E3023]

fn main():
    LOG.push(owns String("a"))
    note(LOG[0])    #$ error[E3031]
    clock = Clock()
    tick(clock)
