#$ test: run-pass
#$ rules: R4
#$ stdout: bye
#$ stdout: 0
## R4, verdict 53: `note` uses `s` after `LOG.clear()`, but `"bye"` is text written in the program, no
## part of `LOG`, so the call is allowed (the verdict's `note("bye")`).

static mut LOG: Array[String] = []

fn note(s: StrView) changes LOG:
    LOG.clear()
    println(s)

fn main() changes LOG:
    LOG.push(String("hello"))
    note("bye")
    println(LOG.len())
