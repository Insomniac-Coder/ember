#$ test: run-pass
#$ rules: R8, R4
#$ stdout: 2
## R8 (g): a view whose last use is an argument of a call does not live across that call; inside it,
## the parameter makes the promise (R4), and the compiler holds `note` to it (verdict 53): `note` uses `s`
## (`String(s)`) before its push to `LOG`, so `note(w)` is allowed though `w` points into `LOG`.

static mut LOG: Array[String] = []

fn note(s: StrView) changes LOG:
    LOG.push(String(s))

fn main() changes LOG:
    LOG.push(String("start"))
    w = LOG[0]
    note(w)
    println(LOG.len())
