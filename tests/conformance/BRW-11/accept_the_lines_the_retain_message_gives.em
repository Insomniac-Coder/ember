# Clause BRW-11.3: "drop the lines first and push the notes after"
#$ test: run-pass
#$ rules: BRW-11, CLO-17
#$ stdout: a
#$ stdout: dropped an empty line
# The message's lines compile and do what `keep` meant.

static mut LOG: Array[String] = []

fn main():
    LOG.push(owns String(""))
    LOG.push(owns String("a"))
    before = LOG.len()
    LOG.retain(owns fn(s) => s != "")
    dropped = before - LOG.len()
    for k in 0..dropped:
        LOG.push(owns String("dropped an empty line"))
    for line in LOG:
        println(line)
