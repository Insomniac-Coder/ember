#$ test: run-pass
#$ rules: R4, R8
#$ stdout: hello
#$ stdout: 0
## R4, verdict 53: a `note` that prints `s` before it clears `LOG` uses `s` after no change, so
## `note(LOG[0])` is allowed: the fact comes from the body, not from the first line, which is the same
## as in the refused program (the owner's own counter-example to checking only the `fn` line).

static mut LOG: Array[String] = []

fn note(s: StrView) changes LOG:
    println(s)
    LOG.clear()

fn main() changes LOG:
    LOG.push(String("hello"))
    note(LOG[0])
    println(LOG.len())
