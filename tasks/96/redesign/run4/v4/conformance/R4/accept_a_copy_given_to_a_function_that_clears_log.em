#$ test: run-pass
#$ rules: R4, R3
#$ stdout: hello
#$ stdout: 0
## R4, verdict 53: the nearest allowed program, the line E3031's message gives: `s` views a copy of
## `LOG[0]` that lives until the end of the line, which `LOG.clear()` does not touch.

static mut LOG: Array[String] = []

fn note(s: StrView) changes LOG:
    LOG.clear()
    println(s)

fn main() changes LOG:
    LOG.push(String("hello"))
    note(copy of LOG[0])
    println(LOG.len())
