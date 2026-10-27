#$ test: run-pass
#$ rules: R8, R12
#$ stdout:
#$ ping
#$ start
## R8 (c), R12: a plain `fn()` changes nothing at all, so calling it inside a loop over `LOG` is allowed
## (verdict 17).

static mut LOG: Array[String] = []

fn replay(hook: fn()):
    for line in LOG:
        hook()
        println(line)

fn ping():
    println("ping")

fn main() changes LOG:
    LOG.push(String("start"))
    replay(ping)
