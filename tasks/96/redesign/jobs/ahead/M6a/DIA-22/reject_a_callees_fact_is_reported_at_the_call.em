# Clause DIA-22.1: "A diagnostic caused by a callee's contract, its signature or a fact the compiler
# worked out from its body (`[BLD-14]`) is reported at the call site in the programmer's code, with
# the callee's declaration as a secondary span."
#$ test: compile-fail
#$ rules: DIA-22, BRW-21, BRW-18, FN-2a
# Two facts from bodies (`note` ends what `s` views; `tick` pushes to `LOG`) and one from a
# signature (`bump` takes `ref mut`): each is claimed on the call's line, none inside the callee.
# The secondary span's label is not in the spec: not claimed.

static mut LOG: Array[String] = []

fn note(s: StrView):
    LOG.clear()
    println(s)

fn tick():
    LOG.push(owns String("tick"))

fn bump(ref mut n: int):
    n += 1

fn main():
    LOG.push(owns String("a"))
    note(LOG[0])    #$ error[E3031]
    for line in LOG:
        println(line)
        tick()    #$ error[E3020]
    a = 1
    bump(a)    #$ error[E3028]
