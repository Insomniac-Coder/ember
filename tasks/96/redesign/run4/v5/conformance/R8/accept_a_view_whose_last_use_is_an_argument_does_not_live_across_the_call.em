#$ test: run-pass
#$ rules: BRW-16, FN-1
#$ stdout: 2
## design rules: R8, R4
## R8 (g): a view whose last use is an argument of a call does not live across that call; inside it,
## the parameter makes the promise (R4), and the compiler holds `note` to it (verdict 53): `note` uses `s`
## (`String(s)`) before its push to `LOG`, so `note(w)` is allowed though `w` points into `LOG`.

static mut LOG: Array[String] = []

fn note(s: StrView):
    LOG.push(String(s))

fn main():
    LOG.push(String("start"))
    w = LOG[0]
    note(w)
    println(LOG.len())
