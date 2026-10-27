#$ test: run-pass
#$ rules: R4, R12
#$ stdout: 2
#$ stdout: dropped an empty line
## R4, R12: the nearest allowed program, the lines the E3022 message gives: the test given to
## `retain` changes nothing, and the notes are pushed after the walk.

static mut LOG: Array[String] = []

fn main():
    LOG.push(String("a"))
    LOG.push(String(""))
    before = LOG.len()
    LOG.retain(fn(s) => s != "")
    dropped = before - LOG.len()
    for k in 0..dropped:
        LOG.push(String("dropped an empty line"))
    println(LOG.len())
    println(LOG[1])
