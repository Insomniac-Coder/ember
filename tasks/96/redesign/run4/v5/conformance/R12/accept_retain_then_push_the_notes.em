#$ test: run-pass
#$ rules: R12, R4
#$ stdout: 2
## R12 (rules.md lines 720-722): the lines the E3022 message of the nearest refused program gives
## (reject_retain_given_a_function_that_changes_the_list); the lambda given to `retain` changes nothing,
## as the compiler works out from its body, so it overlaps nothing.

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
