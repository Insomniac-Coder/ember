#$ test: run-pass
#$ rules: FN-1, OWN-8
#$ stdout: hello
#$ stdout: 0
## design rules: R4, R3
## R4, verdict 53: the nearest allowed program, the line E3031's message gives: `s` views a copy of
## `LOG[0]` that lives until the end of the line, which `LOG.clear()` does not touch.

static mut LOG: Array[String] = []

fn note(s: StrView):
    LOG.clear()
    println(s)

fn main():
    LOG.push(String("hello"))
    note(copy of LOG[0])
    println(LOG.len())
