# Clause BCK-8.2: "Summaries are the least fixed point over the whole program's call graph, so
# recursion is covered and the result depends on nothing but the program"
#$ test: compile-fail
#$ rules: BCK-8, EFF-1, EFF-24, BRW-18
# `ping` and `pong` call each other; only `pong` clears `LOG`, at the bottom of the recursion, and
# the fixed point gives `ping` that change. No text in the spec: code only.

static mut LOG: Array[String] = []

fn ping(n: int):
    if n > 0:
        pong(n - 1)

fn pong(n: int):
    if n == 0:
        LOG.clear()
    else:
        ping(n - 1)

fn main():
    LOG.push(owns String("start"))
    for line in LOG:
        println(line)
        ping(3)    #$ error[E3020]
