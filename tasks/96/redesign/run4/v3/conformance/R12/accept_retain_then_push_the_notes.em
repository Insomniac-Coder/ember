#$ test: run-pass
#$ rules: R12, R4
#$ stdout: 2
## R12 (rules.md lines 556-559): the lines the E3022 message of the nearest refused program gives; the
## lambda given to `retain` changes nothing, so it overlaps nothing.

static mut LOG: Array[String] = []

fn main() changes LOG:
    LOG.push(String("a"))
    LOG.push(String(""))
    before = LOG.len()
    LOG.retain(fn(s) => s != "")
    dropped = before - LOG.len()
    for k in 0..dropped:
        LOG.push(String("dropped an empty line"))
    println(LOG.len())
