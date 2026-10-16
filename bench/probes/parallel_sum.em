## Prices ADR-190's `@parallel` loop: a sum over 400 million turns, each a few integer
## operations, run in chunks on the job system and folded in chunk order. Twin: `parallel_sum.c`
## (the same loop split by hand into one even piece per logical processor, one thread each).

fn main():
    n = 400000000
    total = 0
    @parallel(reduce=[total: +])
    for i in 0..n:
        total += (i * 2654435761) % 1000
    println(total)
